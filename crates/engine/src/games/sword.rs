//! Cut each falling block the way the arrow on it says.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;
use crate::shape;

pub const SPEC: Spec = Spec {
    name: "sword",
    title: "Slanted Sword",
    brief: "Blocks drop onto the block below and sit there waiting. Each carries an arrow \
            pointing one of twelve ways around the clock. Hold the left button and drag \
            through the block the way its arrow points, and it falls in two and the next \
            one drops. Cut one the wrong way and the round is over. Ten of them finish it.",
    controls: &["hold the left button and drag through the block", "release to finish the cut"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Aim, Tag::Drag],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Sword::new(seed))
}

const QUOTA: u32 = 10;
pub(crate) const WAYS: usize = 12;
const SIDE: f32 = 168.0;
const FALL: f32 = 920.0;
const SHELF: f32 = 404.0;
const REST: f32 = SHELF - SIDE * 0.5;
/// How far off the arrow a cut may be. Half of one step around the clock is 15
/// degrees, so this leaves a little short of that either side.
pub(crate) const SLACK: f32 = 0.22;
const SHORTEST: f32 = 46.0;
const SPLIT: u32 = 27;

pub(crate) fn way_of(step: usize) -> (f32, f32) {
    // Twelve o'clock points up the screen, and the numbers run clockwise.
    let angle = step as f32 * std::f32::consts::TAU / WAYS as f32 - std::f32::consts::FRAC_PI_2;
    let (sin, cos) = libm::sincosf(angle);
    (cos, sin)
}

/// Whether a drag is long enough to be a stroke at all. Shorter than this it
/// is a click or a slip of the hand, and no cut was attempted.
pub(crate) fn long_enough(a: (f32, f32), b: (f32, f32)) -> bool {
    libm::hypotf(b.0 - a.0, b.1 - a.1) >= SHORTEST
}

/// Whether a stroke from `a` to `b` cuts the way `step` asks for.
pub(crate) fn cuts_the_way(a: (f32, f32), b: (f32, f32), step: usize) -> bool {
    if !long_enough(a, b) {
        return false;
    }
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let reach = libm::hypotf(dx, dy);
    let (wx, wy) = way_of(step);
    let along = (dx * wx + dy * wy) / reach;
    along >= libm::cosf(SLACK)
}

/// The half of the square on the `keep` side of a cut through its middle, as
/// the corners of a polygon.
fn half(center: (f32, f32), way: (f32, f32), keep: f32) -> Vec<[f32; 2]> {
    let r = SIDE * 0.5;
    let corners = [
        [center.0 - r, center.1 - r],
        [center.0 + r, center.1 - r],
        [center.0 + r, center.1 + r],
        [center.0 - r, center.1 + r],
    ];
    let normal = (-way.1 * keep, way.0 * keep);
    let side = |p: &[f32; 2]| (p[0] - center.0) * normal.0 + (p[1] - center.1) * normal.1;
    let mut out = Vec::new();
    for slot in 0..4 {
        let (here, next) = (corners[slot], corners[(slot + 1) % 4]);
        let (a, b) = (side(&here), side(&next));
        if a >= 0.0 {
            out.push(here);
        }
        if (a >= 0.0) != (b >= 0.0) {
            let t = a / (a - b);
            out.push([here[0] + (next[0] - here[0]) * t, here[1] + (next[1] - here[1]) * t]);
        }
    }
    out
}

enum Block {
    Dropping { at: f32, step: usize },
    Resting { step: usize },
    Split { step: usize, age: u32 },
}

struct Sword {
    rng: Rng,
    block: Block,
    drag: Option<(f32, f32)>,
    cut: u32,
}

impl Sword {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let step = rng.below(WAYS);
        Sword { rng, block: Block::Dropping { at: -SIDE * 0.5, step }, drag: None, cut: 0 }
    }

    fn drop_next(&mut self) {
        let step = self.rng.below(WAYS);
        self.block = Block::Dropping { at: -SIDE * 0.5, step };
    }
}

fn middle() -> f32 {
    frame::WIDTH as f32 * 0.5
}

/// The block, and the arrow saying which way it wants cutting.
fn face(frame: &mut Frame, at: f32, step: usize) {
    let r = SIDE * 0.5;
    frame.bordered(middle() - r, at - r, SIDE, SIDE, 4.0, palette::PANEL, palette::EDGE);
    let (wx, wy) = way_of(step);
    let reach = 54.0;
    let tip = (middle() + wx * reach, at + wy * reach);
    let tail = (middle() - wx * reach, at - wy * reach);
    frame.line(tail.0, tail.1, tip.0, tip.1, 9.0, palette::WARN);
    for turn in [-1.0f32, 1.0] {
        let wing = (
            tip.0 + (-wx * 0.7 - wy * 0.5 * turn) * 44.0,
            tip.1 + (-wy * 0.7 + wx * 0.5 * turn) * 44.0,
        );
        frame.line(tip.0, tip.1, wing.0, wing.1, 9.0, palette::WARN);
    }
}

/// Whether a stroke passes through the square at all.
fn crosses(a: (f32, f32), b: (f32, f32), center: (f32, f32)) -> bool {
    shape::stroke_hits(a, b, center, (SIDE * 0.5, SIDE * 0.5))
}

impl Game for Sword {
    fn update(&mut self, input: &Input) -> Outcome {
        match self.block {
            Block::Dropping { at, step } => {
                let at = at + FALL * TICK;
                self.block =
                    if at >= REST { Block::Resting { step } } else { Block::Dropping { at, step } };
            }
            Block::Split { step, age } => {
                let age = age + 1;
                if age >= SPLIT {
                    self.cut += 1;
                    if self.cut == QUOTA {
                        return Outcome::Won;
                    }
                    self.drop_next();
                } else {
                    self.block = Block::Split { step, age };
                }
            }
            Block::Resting { step } => {
                let now = input.cursor();
                if input.button_pressed(Button::Left) {
                    self.drag = Some(now);
                }
                // A drag too short to be a stroke is let go of the same way a
                // stroke that misses the block is. Only a real stroke the
                // wrong way ends the round.
                if input.button_released(Button::Left)
                    && let Some(from) = self.drag.take()
                    && long_enough(from, now)
                    && crosses(from, now, (middle(), REST))
                {
                    if !cuts_the_way(from, now, step) {
                        return Outcome::Lost;
                    }
                    self.block = Block::Split { step, age: 0 };
                }
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        frame.rect(0.0, SHELF, frame::WIDTH as f32, frame::HEIGHT as f32 - SHELF, palette::PANEL);
        frame.rect(0.0, SHELF, frame::WIDTH as f32, 4.0, palette::EDGE);
        match self.block {
            Block::Dropping { at, step } => face(frame, at, step),
            Block::Resting { step } => face(frame, REST, step),
            Block::Split { step, age } => {
                let (wx, wy) = way_of(step);
                let spread = 150.0 * age as f32 * TICK;
                for keep in [-1.0f32, 1.0] {
                    let shove = (-wy * keep * spread, wx * keep * spread);
                    let piece: Vec<[f32; 2]> = half((middle(), REST), (wx, wy), keep)
                        .into_iter()
                        .map(|p| [p[0] + shove.0, p[1] + shove.1])
                        .collect();
                    frame.poly(&piece, palette::EDGE);
                }
            }
        }
        if let Some(from) = self.drag {
            frame.disc(from.0, from.1, 5.0, palette::MARK);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/sword.rs"]
mod tests;
