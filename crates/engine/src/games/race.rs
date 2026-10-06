//! One lap of a track, steering a car that only knows left and right.

use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "race",
    title: "Raceway Royale",
    brief: "Drive one lap. SPACE is the throttle and there is no brake: let go of it and the \
            car runs down on its own. LEFT and RIGHT do not steer the car about the screen, \
            they steer the car about itself, so which way the nose is pointing is the only \
            thing that says what they will do. The gray is the road and the green beside it \
            is grass, which will not let you go faster than a crawl. The walls beyond the \
            grass stop you dead, and there is no way across the middle. Pass all the marks \
            in order and come back to the line.",
    controls: &[
        "hold SPACE for the throttle, let go to run down",
        "LEFT and RIGHT turn the car itself",
    ],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Friendly, Tag::Navigate, Tag::Physics, Tag::Overhead, Tag::Slippery],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Race::new())
}

/// The middle of the road, all the way around. Everything else about the track
/// is how far from this line a place is.
pub(crate) const TRACK: [[f32; 2]; 12] = [
    [200.0, 80.0],
    [550.0, 80.0],
    [550.0, 240.0],
    [550.0, 400.0],
    [400.0, 400.0],
    [400.0, 260.0],
    [320.0, 260.0],
    [240.0, 260.0],
    [240.0, 400.0],
    [90.0, 400.0],
    [90.0, 240.0],
    [90.0, 80.0],
];

pub(crate) const ROAD: f32 = 50.0;
pub(crate) const VERGE: f32 = 92.0;
/// Four marks around the track, to be passed in order: the line, then one on
/// each of three straights.
const MARKS: usize = 4;
const MARK_NODES: [usize; MARKS] = [0, 2, 6, 10];
const MARK_NEAR: f32 = 60.0;

const CAR: f32 = 9.0;
const THROTTLE: f32 = 340.0;
const DRAG: f32 = 1.6;
const TOP: f32 = 190.0;
/// How fast the car may go once it is off the road.
const CRAWL: f32 = 62.0;
const SWING: f32 = 2.8;

type Spot = (f32, f32);

/// How far a place is from the middle of the road.
pub(crate) fn off_line(at: Spot) -> f32 {
    let mut nearest = f32::MAX;
    for i in 0..TRACK.len() {
        let a = (TRACK[i][0], TRACK[i][1]);
        let b = (TRACK[(i + 1) % TRACK.len()][0], TRACK[(i + 1) % TRACK.len()][1]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let along = dx * dx + dy * dy;
        let part = if along == 0.0 {
            0.0
        } else {
            (((at.0 - a.0) * dx + (at.1 - a.1) * dy) / along).clamp(0.0, 1.0)
        };
        let off = libm::hypotf(at.0 - (a.0 + dx * part), at.1 - (a.1 + dy * part));
        nearest = nearest.min(off);
    }
    nearest
}

/// Which way is straight out from the road here, taken from how the distance
/// to the middle of it changes on either side of the spot.
fn wall_way(at: Spot) -> Spot {
    let out = (
        off_line((at.0 + 1.0, at.1)) - off_line((at.0 - 1.0, at.1)),
        off_line((at.0, at.1 + 1.0)) - off_line((at.0, at.1 - 1.0)),
    );
    let long = libm::hypotf(out.0, out.1);
    if long == 0.0 { (0.0, 0.0) } else { (out.0 / long, out.1 / long) }
}

fn node_of(number: usize) -> usize {
    MARK_NODES[number]
}

pub(crate) fn mark_at(number: usize) -> Spot {
    let point = TRACK[node_of(number)];
    (point[0], point[1])
}

/// Which way the road runs where a mark stands, taken from the points either
/// side of it so that the line drawn across the road sits square to it.
fn road_way(number: usize) -> f32 {
    let node = node_of(number);
    let before = TRACK[(node + TRACK.len() - 1) % TRACK.len()];
    let after = TRACK[(node + 1) % TRACK.len()];
    libm::atan2f(after[1] - before[1], after[0] - before[0])
}

pub(crate) struct Race {
    pub(crate) at: Spot,
    pub(crate) facing: f32,
    pub(crate) pace: f32,
    pub(crate) passed: usize,
}

impl Race {
    pub(crate) fn new() -> Self {
        let start = mark_at(0);
        let next = (TRACK[1][0], TRACK[1][1]);
        Race {
            at: start,
            facing: libm::atan2f(next.1 - start.1, next.0 - start.0),
            pace: 0.0,
            passed: 0,
        }
    }
}

impl Game for Race {
    fn update(&mut self, input: &Input) -> Outcome {
        self.facing += input.axis(Key::Left, Key::Right) * SWING * TICK;
        let throttle = if input.held(Key::Space) { THROTTLE } else { 0.0 };
        self.pace += (throttle - self.pace * DRAG) * TICK;
        let ceiling = if off_line(self.at) > ROAD * 0.5 { CRAWL } else { TOP };
        self.pace = self.pace.min(ceiling);

        let (sin, cos) = libm::sincosf(self.facing);
        let step = self.pace * TICK;
        let clear = |at: (f32, f32)| off_line(at) + CAR <= VERGE * 0.5;
        let goal = (self.at.0 + cos * step, self.at.1 + sin * step);
        if clear(goal) {
            self.at = goal;
        } else {
            // A car that cannot go the way it is pointing scrapes along the
            // barrier instead of sticking to it, which is what lets one nosed
            // into a wall be driven out again.
            let out = wall_way(self.at);
            let into = cos * out.0 + sin * out.1;
            let along = (cos - out.0 * into, sin - out.1 * into);
            let slid = (self.at.0 + along.0 * step, self.at.1 + along.1 * step);
            if clear(slid) {
                self.at = slid;
                self.pace *= 0.9;
            } else {
                self.pace = 0.0;
            }
        }

        let next = mark_at((self.passed + 1) % MARKS);
        if libm::hypotf(self.at.0 - next.0, self.at.1 - next.1) < MARK_NEAR {
            self.passed += 1;
            if self.passed == MARKS {
                return Outcome::Won;
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let grass = Rgb(38, 72, 44);
        let tarmac = Rgb(52, 54, 62);
        for (band, tint) in [(VERGE + 10.0, palette::MUTED), (VERGE, grass), (ROAD, tarmac)] {
            for i in 0..TRACK.len() {
                let a = (TRACK[i][0], TRACK[i][1]);
                let b = (TRACK[(i + 1) % TRACK.len()][0], TRACK[(i + 1) % TRACK.len()][1]);
                frame.line(a.0, a.1, b.0, b.1, band, tint);
                frame.disc(a.0, a.1, band * 0.5, tint);
            }
        }

        // The line to come back to, and the marks still to be passed, so the
        // way around is on the screen rather than in the rules.
        for number in 0..MARKS {
            let at = mark_at(number);
            let facing = road_way(number);
            let (sin, cos) = libm::sincosf(facing);
            let half = ROAD * 0.5 + 5.0;
            let tint = if number == 0 {
                palette::MARK
            } else if number <= self.passed {
                palette::EDGE
            } else {
                palette::GOOD
            };
            frame.line(
                at.0 + sin * half,
                at.1 - cos * half,
                at.0 - sin * half,
                at.1 + cos * half,
                4.0,
                tint,
            );
            // The arrow stands a little past the line rather than on it, so
            // that the two do not read as one mark.
            let ahead = (at.0 + cos * 20.0, at.1 + sin * 20.0);
            frame.spun(&[[9.0, 0.0], [-5.0, -7.0], [-5.0, 7.0]], ahead, facing, tint);
        }

        let body = [[10.0, -6.0], [10.0, 6.0], [-9.0, 7.0], [-9.0, -7.0]];
        frame.spun(&body, self.at, self.facing, palette::PLAYER);
        let nose = [[12.0, 0.0], [4.0, -6.0], [4.0, 6.0]];
        frame.spun(&nose, self.at, self.facing, palette::WARN);
    }
}

#[cfg(test)]
#[path = "../tests/games/race.rs"]
mod tests;
