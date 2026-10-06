//! Moving a body about a place that has walls in it.

use crate::game::TICK;

/// A full stride the way asked for, or along whichever half of that way is not
/// walled off, so that turning a corner costs no ground. `free` says whether
/// the body may stand with its middle at a point.
pub fn walked(
    at: (f32, f32),
    step: (f32, f32),
    reach: f32,
    free: impl Fn(f32, f32) -> bool,
) -> (f32, f32) {
    let (across, down) = (step.0 * reach, step.1 * reach);
    if free(at.0 + across, at.1 + down) {
        return (at.0 + across, at.1 + down);
    }
    if step.0 != 0.0 && free(at.0 + step.0.signum() * reach, at.1) {
        return (at.0 + step.0.signum() * reach, at.1);
    }
    if step.1 != 0.0 && free(at.0, at.1 + step.1.signum() * reach) {
        return (at.0, at.1 + step.1.signum() * reach);
    }
    at
}

/// Where a step along one axis ends: the whole step, or right up against
/// whatever stopped it, so that running into a wall never leaves a gap for the
/// next turn to jump across. `blocked` asks about a place along that axis.
///
/// A step must be no longer than a cell. What stops the body is taken to be
/// the cell its leading edge came to rest in, which is only the first cell in
/// the way while nothing is fast enough to cross a whole one in a tick.
pub fn slid(along: f32, step: f32, half: f32, cell: f32, blocked: impl Fn(f32) -> bool) -> f32 {
    if !blocked(along + step) {
        return along + step;
    }
    // A hair clear of the wall, so that the body is not counted inside it.
    const NUDGE: f32 = 0.01;
    if step > 0.0 {
        (((along + step + half) / cell).floor() * cell - half - NUDGE).max(along)
    } else {
        (((along + step - half) / cell).floor() * cell + cell + half + NUDGE).min(along)
    }
}

/// Pixels a second a body that runs and jumps runs at.
const RUN: f32 = 150.0;
const GRAVITY: f32 = 1400.0;
/// The speed a jump leaves the ground at.
const JUMP: f32 = 440.0;
/// The fastest a body falls.
const FALL: f32 = 600.0;

/// A body that runs and jumps about a room of cells: where it is, and how
/// fast it is falling, going up being negative.
#[derive(Clone, Copy)]
pub struct Hopper {
    pub at: (f32, f32),
    pub climb: f32,
}

impl Hopper {
    /// Whether it is on the ground, which is whether a hair below it is solid.
    pub fn standing(self, blocked: impl Fn((f32, f32)) -> bool) -> bool {
        blocked((self.at.0, self.at.1 + 1.0))
    }

    /// One tick of running `across` one way or the other or not at all, and
    /// of jumping if asked to while standing. `half` is the body's half side,
    /// `cell` the room's, and `blocked` whether the body would overlap
    /// something solid with its middle at a point.
    pub fn stepped(
        self,
        across: f32,
        jump: bool,
        half: f32,
        cell: f32,
        blocked: impl Fn((f32, f32)) -> bool,
    ) -> Hopper {
        let (mut x, mut y) = self.at;
        let mut climb = if jump && self.standing(&blocked) { -JUMP } else { self.climb };
        climb = (climb + GRAVITY * TICK).min(FALL);
        x = slid(x, across * RUN * TICK, half, cell, |x| blocked((x, y)));
        let fell = y + climb * TICK;
        y = slid(y, climb * TICK, half, cell, |y| blocked((x, y)));
        if y != fell {
            climb = 0.0;
        }
        Hopper { at: (x, y), climb }
    }
}

/// A body moved `step` toward `goal`, landing exactly on it when it is within
/// a step. What is left of the way is never overshot.
pub fn toward(at: (f32, f32), goal: (f32, f32), step: f32) -> (f32, f32) {
    let (dx, dy) = (goal.0 - at.0, goal.1 - at.1);
    let away = libm::hypotf(dx, dy);
    if away <= step || away == 0.0 {
        return goal;
    }
    (at.0 + dx / away * step, at.1 + dy / away * step)
}

/// The same along one axis: eased `step` toward `goal`, settling on it rather
/// than flicking past.
pub fn eased(along: f32, goal: f32, step: f32) -> f32 {
    if (goal - along).abs() <= step { goal } else { along + step * (goal - along).signum() }
}

/// A body reaching `radius` on either side of `spot` that has run past a wall,
/// put back inside and turned around, keeping `give` of the speed it arrived
/// with. A `give` of nothing stops it dead against the wall instead.
pub fn bounced(spot: &mut f32, pace: &mut f32, low: f32, high: f32, radius: f32, give: f32) {
    if *spot - radius < low || *spot + radius > high {
        *spot = spot.clamp(low + radius, high - radius);
        *pace = -*pace * give;
    }
}

/// Two round bodies of the same weight that overlap, pushed apart by half the
/// overlap each and, where they are closing, bounced off one another keeping
/// `give` of the speed they met at.
pub fn bumped(
    one: (&mut (f32, f32), &mut (f32, f32)),
    two: (&mut (f32, f32), &mut (f32, f32)),
    radius: f32,
    give: f32,
) {
    let (at_one, pace_one) = one;
    let (at_two, pace_two) = two;
    let (dx, dy) = (at_two.0 - at_one.0, at_two.1 - at_one.1);
    let apart = libm::hypotf(dx, dy);
    if apart >= radius * 2.0 || apart <= 0.0 {
        return;
    }
    let way = (dx / apart, dy / apart);
    let over = (radius * 2.0 - apart) * 0.5;
    *at_one = (at_one.0 - way.0 * over, at_one.1 - way.1 * over);
    *at_two = (at_two.0 + way.0 * over, at_two.1 + way.1 * over);
    let closing = (pace_two.0 - pace_one.0) * way.0 + (pace_two.1 - pace_one.1) * way.1;
    if closing < 0.0 {
        let shove = -(1.0 + give) * closing * 0.5;
        *pace_one = (pace_one.0 - way.0 * shove, pace_one.1 - way.1 * shove);
        *pace_two = (pace_two.0 + way.0 * shove, pace_two.1 + way.1 * shove);
    }
}
