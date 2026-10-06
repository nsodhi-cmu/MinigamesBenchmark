//! Keep away from something slower that walks straight through everything you cannot.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::Input;
use crate::palette;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "ghost",
    title: "Ghost Tag",
    brief: "A ghost drifts straight at you across a park of ramps, blocks, and bowls, and it \
            goes through all of them as if they were not there. You cannot: they stop you. \
            You are much the faster of the two, so the danger is not its speed but the corner \
            it puts you in. Keep away from it for thirty seconds. Let it touch you and the \
            round is over.",
    controls: &["arrow keys skate, diagonals too"],
    pointer: false,
    scoring: Scoring::Boolean,
    varies: false,
    tags: &[Tag::Fixed, Tag::Lethal, Tag::Navigate, Tag::Overhead],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Ghost::new())
}

pub(crate) const SURVIVE: u32 = 30 * TICK_HZ;
pub(crate) const PACE: f32 = 250.0;
pub(crate) const DRIFT: f32 = 132.0;
pub(crate) const BODY: f32 = 10.0;
const SPOOK: f32 = 13.0;
const TOP: f32 = 4.0;
const BOTTOM: f32 = frame::HEIGHT as f32 - 4.0;
const SIDE: f32 = 4.0;

/// The blocks of the park: left, top, width, height.
pub(crate) const BLOCKS: &[(f32, f32, f32, f32)] = &[
    (120.0, 96.0, 150.0, 26.0),
    (250.0, 190.0, 26.0, 130.0),
    (60.0, 240.0, 120.0, 26.0),
    (330.0, 60.0, 26.0, 110.0),
    (390.0, 250.0, 170.0, 26.0),
    (330.0, 360.0, 26.0, 90.0),
    (450.0, 326.0, 120.0, 26.0),
    (100.0, 400.0, 130.0, 26.0),
];

/// The bowls of the park, which are as solid as the blocks: middle and radius.
pub(crate) const BOWLS: &[(f32, f32, f32)] =
    &[(170.0, 178.0, 30.0), (470.0, 140.0, 50.0), (180.0, 330.0, 30.0), (560.0, 416.0, 30.0)];

const RAMP: Rgb = Rgb(110, 116, 138);
const RAMP_EDGE: Rgb = Rgb(150, 156, 178);
const SPECTER: Rgb = Rgb(216, 224, 255);

/// Whether a player of radius `BODY` standing at `at` is inside anything solid.
pub(crate) fn solid(at: (f32, f32)) -> bool {
    if at.0 < SIDE + BODY
        || at.0 > frame::WIDTH as f32 - SIDE - BODY
        || at.1 < TOP + BODY
        || at.1 > BOTTOM - BODY
    {
        return true;
    }
    let in_block = BLOCKS.iter().any(|(x, y, w, h)| {
        let near = (at.0.clamp(*x, x + w), at.1.clamp(*y, y + h));
        (near.0 - at.0).powi(2) + (near.1 - at.1).powi(2) < BODY * BODY
    });
    in_block
        || BOWLS
            .iter()
            .any(|(x, y, r)| (at.0 - x).powi(2) + (at.1 - y).powi(2) < (r + BODY) * (r + BODY))
}

pub(crate) struct Ghost {
    age: u32,
    pub(crate) at: (f32, f32),
    pub(crate) ghost: (f32, f32),
}

impl Ghost {
    pub(crate) fn new() -> Self {
        Ghost { age: 0, at: (70.0, 430.0), ghost: (570.0, 70.0) }
    }

    fn caught(&self) -> bool {
        let (dx, dy) = (self.at.0 - self.ghost.0, self.at.1 - self.ghost.1);
        dx * dx + dy * dy < (BODY + SPOOK * 0.6).powi(2)
    }
}

impl Game for Ghost {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some(step) = input.heading() {
            self.at = walk::walked(self.at, step, PACE * TICK, |x, y| !solid((x, y)));
        }

        self.ghost = walk::toward(self.ghost, self.at, DRIFT * TICK);
        if self.caught() {
            return Outcome::Lost;
        }

        self.age += 1;
        if self.age >= SURVIVE { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.rect(0.0, 0.0, frame::WIDTH as f32, frame::HEIGHT as f32, palette::PANEL);
        for (x, y, r) in BOWLS {
            frame.disc(*x, *y, *r, palette::EDGE);
            frame.ring(*x, *y, *r, 3.0, RAMP_EDGE);
            frame.ring(*x, *y, r * 0.55, 2.0, RAMP);
        }
        for (x, y, w, h) in BLOCKS {
            frame.bordered(*x, *y, *w, *h, 2.0, RAMP, RAMP_EDGE);
        }

        frame.disc(self.at.0, self.at.1, BODY, palette::PLAYER);
        let (gx, gy) = self.ghost;
        frame.disc(gx, gy - 2.0, SPOOK, SPECTER);
        frame.rect(gx - SPOOK, gy - 2.0, SPOOK * 2.0, SPOOK * 0.9, SPECTER);
        frame.rect(gx - 5.0, gy - 5.0, 3.0, 4.0, palette::PANEL);
        frame.rect(gx + 2.0, gy - 5.0, 3.0, 4.0, palette::PANEL);
    }
}

#[cfg(test)]
#[path = "../tests/games/ghost.rs"]
mod tests;
