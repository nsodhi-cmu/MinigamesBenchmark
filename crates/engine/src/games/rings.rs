//! Turn a round maze so that the ball in the middle of it rolls its way out, cell by cell.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "rings",
    title: "Rolling Rings",
    brief: "A ball sits in the middle of a round maze of five rings, each cut into sixteen \
            cells by the spokes between them. Hold LEFT or RIGHT to turn the whole maze at a \
            steady speed. The ball rolls around its ring toward the bottom of the screen and \
            stops where a spoke is in its way. A gap in the wall beneath it drops it out to \
            the next ring, and a gap in the wall above it, when a spoke is holding it up near \
            the top, drops it back in. Get the ball out through the gap in the outermost \
            wall.",
    controls: &["LEFT turns the maze counterclockwise", "RIGHT turns the maze clockwise"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Friendly, Tag::Puzzle, Tag::Physics],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Rings::new())
}

pub(crate) const RINGS: usize = 5;
pub(crate) const SECTORS: usize = 16;

/// A maze: where each wall is open, from the wall around the middle out, bit
/// `s` for cell `s`; and where each ring has a spoke, bit `s` for the spoke
/// between cell `s` and the next.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Maze {
    pub(crate) open: [u16; RINGS + 1],
    pub(crate) spokes: [u16; RINGS],
}

pub(crate) const MAZE: Maze = Maze {
    open: [
        0b0011000110000000,
        0b0000000001000000,
        0b0000010000010010,
        0b0000100000000000,
        0b0001000010000001,
        0b0010000000000000,
    ],
    spokes: [
        0b0000000100010000,
        0b0001100001000000,
        0b0000001001000110,
        0b0100001000000000,
        0b0001000001000101,
    ],
};

const MIDDLE: (f32, f32) = (frame::WIDTH as f32 * 0.5, frame::HEIGHT as f32 * 0.5);
pub(crate) const BALL: f32 = 7.0;
/// Angles are counted in tenths of a degree, so that where the ball is and
/// how far the maze is turned are whole numbers a search can tell apart.
pub(crate) const WHOLE: i32 = 3600;
const SECTOR: i32 = WHOLE / SECTORS as i32;
/// Tenths of a degree a tick the maze turns while an arrow is held.
pub(crate) const SPIN: i32 = 15;
/// How fast the ball rolls along a ring tipped right over, in pixels a second.
/// Tipped less it rolls slower in proportion, so that it rolls whenever the
/// ring under it is not level and settles at the bottom without swinging past.
const PACE: f32 = 420.0;
/// How far below level a gap has to be before the ball drops out through it,
/// or above level to drop in, as the sine of its angle: a few degrees, since
/// below level the ball rests against the outer wall and above it the inner.
const DROPS: f32 = 0.1;
/// Ticks the ball takes to fall through a gap from one ring to the next.
pub(crate) const FALL: u8 = 6;
const WALL: Rgb = Rgb(120, 140, 190);

fn radius(wall: usize) -> f32 {
    30.0 + 36.0 * wall as f32
}

/// How far from the middle the ball runs on a ring: around the inside of the
/// middle cell for ring zero, and past the last wall for out.
pub(crate) fn lane(ring: usize) -> f32 {
    match ring {
        0 => 14.0,
        ring if ring > RINGS => radius(RINGS) + 18.0,
        ring => (radius(ring - 1) + radius(ring)) * 0.5,
    }
}

/// An arc of `pixels` at a radius, in tenths of a degree.
fn arc(pixels: f32, at: f32) -> i32 {
    (libm::asinf(pixels / at).to_degrees() * 10.0).round() as i32
}

fn sector_of(angle: i32) -> usize {
    (angle.rem_euclid(WHOLE) / SECTOR) as usize
}

fn sine(angle: i32) -> f32 {
    libm::sinf((angle as f32 / 10.0).to_radians())
}

fn cosine(angle: i32) -> f32 {
    libm::cosf((angle as f32 / 10.0).to_radians())
}

/// A point of the maze, from its middle, at an angle with the maze turned
/// `turn` clockwise.
fn polar(r: f32, angle: i32, turn: i32) -> (f32, f32) {
    let a = ((angle + turn) as f32 / 10.0).to_radians();
    (r * libm::cosf(a), r * libm::sinf(a))
}

impl Maze {
    fn open(&self, wall: usize, sector: usize) -> bool {
        self.open[wall] >> sector & 1 == 1
    }

    fn spoke(&self, ring: usize, sector: usize) -> bool {
        self.spokes[ring - 1] >> sector & 1 == 1
    }

    /// Whether a ball at an angle fits through a wall there: the wall is open
    /// on both sides of it nearly as far around as the ball reaches, a little
    /// less so that a ball resting on a spoke fits a gap that runs up to it.
    fn fits(&self, wall: usize, angle: i32) -> bool {
        let reach = arc(BALL - 1.0, radius(wall));
        self.open(wall, sector_of(angle - reach)) && self.open(wall, sector_of(angle + reach))
    }

    /// Whether a ball at an angle on a ring is a ball's width clear of every
    /// spoke of the ring, which is where it can rest and where it can land.
    fn clear(&self, ring: usize, angle: i32) -> bool {
        if ring == 0 || ring > RINGS {
            return true;
        }
        let clear = arc(BALL + 1.0, lane(ring));
        (0..SECTORS).filter(|s| self.spoke(ring, *s)).all(|sector| {
            let spoke = (sector as i32 + 1) * SECTOR;
            ((spoke - angle + WHOLE / 2).rem_euclid(WHOLE) - WHOLE / 2).abs() >= clear
        })
    }

    /// Rolled `step` tenths of a degree around a ring from an angle, stopping a
    /// ball's width short of any spoke in the way.
    fn rolled(&self, ring: usize, angle: i32, step: i32) -> i32 {
        if ring == 0 || ring > RINGS {
            return (angle + step).rem_euclid(WHOLE);
        }
        let clear = arc(BALL + 1.0, lane(ring));
        let mut step = step;
        for sector in (0..SECTORS).filter(|s| self.spoke(ring, *s)) {
            let spoke = (sector as i32 + 1) * SECTOR;
            let ahead = (spoke - angle + WHOLE / 2).rem_euclid(WHOLE) - WHOLE / 2;
            if step > 0 && ahead >= 0 {
                step = step.min((ahead - clear).max(0));
            } else if step < 0 && ahead <= 0 {
                step = step.max((ahead + clear).min(0));
            }
        }
        (angle + step).rem_euclid(WHOLE)
    }

    /// One tick of the ball with the maze turned to `turn`: a fall goes on, a
    /// ball over a gap beneath or above it drops through, and otherwise it
    /// rolls around its ring as far as the tilt of the ring under it carries it.
    pub(crate) fn tick(&self, ball: Ball, turn: i32) -> Ball {
        match ball {
            Ball::Out => Ball::Out,
            Ball::Falling { from, to, angle, left } if left > 1 => {
                Ball::Falling { from, to, angle, left: left - 1 }
            }
            Ball::Falling { to, .. } if to > RINGS => Ball::Out,
            Ball::Falling { to, angle, .. } => Ball::At { ring: to, angle },
            Ball::At { ring, angle } => {
                let down = sine(angle + turn);
                let out = ring <= RINGS && down >= DROPS && self.fits(ring, angle);
                if out && self.clear(ring + 1, angle) {
                    return Ball::Falling { from: ring, to: ring + 1, angle, left: FALL };
                }
                let inward = ring >= 1 && down <= -DROPS && self.fits(ring - 1, angle);
                if inward && self.clear(ring - 1, angle) {
                    return Ball::Falling { from: ring, to: ring - 1, angle, left: FALL };
                }
                let along = PACE * cosine(angle + turn) * TICK;
                let step = (along / lane(ring) * (1800.0 / std::f32::consts::PI)).round() as i32;
                Ball::At { ring, angle: self.rolled(ring, angle, step) }
            }
        }
    }
}

/// Where the ball is: on a ring at an angle in the maze's own frame, ring
/// zero being the middle; falling through a gap from one ring to the next;
/// or out.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Ball {
    At { ring: usize, angle: i32 },
    Falling { from: usize, to: usize, angle: i32, left: u8 },
    Out,
}

impl Ball {
    /// Where it is from the middle, with the maze turned `turn` clockwise.
    fn place(self, turn: i32) -> (f32, f32) {
        match self {
            Ball::Out => (0.0, 0.0),
            Ball::At { ring, angle } => polar(lane(ring), angle, turn),
            Ball::Falling { from, to, angle, left } => {
                let along = 1.0 - left as f32 / FALL as f32;
                polar(lane(from) + (lane(to) - lane(from)) * along, angle, turn)
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Rings {
    /// How far the maze is turned clockwise, in tenths of a degree.
    pub(crate) turn: i32,
    pub(crate) ball: Ball,
}

impl Rings {
    pub(crate) fn new() -> Self {
        Rings { turn: 0, ball: Ball::At { ring: 0, angle: WHOLE / 4 } }
    }
}

impl Game for Rings {
    fn update(&mut self, input: &Input) -> Outcome {
        let way = input.axis(Key::Left, Key::Right) as i32;
        self.turn = (self.turn + way * SPIN).rem_euclid(WHOLE);
        self.ball = MAZE.tick(self.ball, self.turn);
        if self.ball == Ball::Out { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.disc(MIDDLE.0, MIDDLE.1, radius(RINGS), palette::PANEL);
        let at = |r: f32, angle: i32| {
            let (x, y) = polar(r, angle, self.turn);
            (MIDDLE.0 + x, MIDDLE.1 + y)
        };
        for (wall, open) in MAZE.open.iter().enumerate() {
            for sector in (0..SECTORS).filter(|s| open >> s & 1 == 0) {
                let from = sector as i32 * SECTOR;
                for step in 0..6 {
                    let (x0, y0) = at(radius(wall), from + step * SECTOR / 6);
                    let (x1, y1) = at(radius(wall), from + (step + 1) * SECTOR / 6);
                    frame.line(x0, y0, x1, y1, 4.0, WALL);
                }
            }
        }
        for (ring, spokes) in MAZE.spokes.iter().enumerate() {
            for sector in (0..SECTORS).filter(|s| spokes >> s & 1 == 1) {
                let a = (sector as i32 + 1) * SECTOR;
                let (x0, y0) = at(radius(ring), a);
                let (x1, y1) = at(radius(ring + 1), a);
                frame.line(x0, y0, x1, y1, 4.0, WALL);
            }
        }
        for sector in (0..SECTORS).filter(|s| MAZE.open[RINGS] >> s & 1 == 1) {
            for side in [sector, sector + 1] {
                let (x0, y0) = at(radius(RINGS) - 6.0, side as i32 * SECTOR);
                let (x1, y1) = at(radius(RINGS) + 8.0, side as i32 * SECTOR);
                frame.line(x0, y0, x1, y1, 4.0, palette::GOOD);
            }
        }

        let (x, y) = self.ball.place(self.turn);
        frame.disc(MIDDLE.0 + x, MIDDLE.1 + y, BALL, palette::WARN);
    }
}

#[cfg(test)]
#[path = "../tests/games/rings.rs"]
mod tests;
