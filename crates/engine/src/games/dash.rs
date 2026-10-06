//! Jump the blocks and duck the bars until the course runs out.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "dash",
    title: "Dynamic Dash",
    brief: "The course runs at you from the right and does not slow down. Jump the blocks \
            sitting on the ground and duck the bars hanging above it. Twenty things stand in it. Reach the end of \
            the course, which is thirty seconds long, to finish. Touch any part of it and \
            the round is over.",
    controls: &[
        "SPACE or UP jumps",
        "each press is one jump, so let go between them",
        "hold DOWN to duck",
    ],
    pointer: false,
    scoring: Scoring::Boolean,
    varies: true,
    tags: &[Tag::Fixed, Tag::Lethal, Tag::Timing],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Vault::new(seed))
}

/// Where the finish line starts out: exactly the ground covered in the round.
fn course() -> f32 {
    RUNNER_X + SCROLL * (LASTS as f32 * TICK)
}

const GROUND: f32 = 400.0;
const RUNNER_X: f32 = 108.0;
const TALL: f32 = 46.0;
const SHORT: f32 = 24.0;
const WIDE: f32 = 26.0;
const SCROLL: f32 = 380.0;
const LIFT: f32 = 560.0;
const GRAVITY: f32 = 1750.0;
const LASTS: u32 = 30 * TICK_HZ;
const HURDLES: usize = 20;

/// A bar hangs low enough that standing hits it and ducking does not.
const BAR_TOP: f32 = 302.0;
const BAR_BOTTOM: f32 = 368.0;

enum Kind {
    Block(f32),
    Bar,
}

struct Hurdle {
    start: f32,
    kind: Kind,
}

struct Vault {
    hurdles: Vec<Hurdle>,
    age: u32,
    height: f32,
    rise: f32,
    ducking: bool,
}

impl Vault {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let opening = frame::WIDTH as f32 + 300.0;
        let spread = (course() - 520.0 - opening) / (HURDLES - 1) as f32;
        let hurdles = (0..HURDLES)
            .map(|slot| {
                let kind =
                    if rng.chance(0.4) { Kind::Bar } else { Kind::Block(rng.range(24.0, 44.0)) };
                let start = opening + slot as f32 * spread + rng.range(-110.0, 110.0);
                Hurdle { start, kind }
            })
            .collect();
        Vault { hurdles, age: 0, height: 0.0, rise: 0.0, ducking: false }
    }

    /// How far the course has run past, decided by the tick count alone.
    fn traveled(&self) -> f32 {
        SCROLL * (self.age as f32 * TICK)
    }

    fn body(&self) -> (f32, f32) {
        let head = if self.ducking && self.height == 0.0 { SHORT } else { TALL };
        (GROUND - self.height - head, head)
    }
}

impl Game for Vault {
    fn update(&mut self, input: &Input) -> Outcome {
        self.ducking = input.held(Key::Down);
        if input.lift_pressed() && self.height == 0.0 {
            self.rise = LIFT;
        }
        if self.rise != 0.0 || self.height > 0.0 {
            self.rise -= GRAVITY * TICK;
            self.height += self.rise * TICK;
            if self.height <= 0.0 {
                self.height = 0.0;
                self.rise = 0.0;
            }
        }

        self.age += 1;
        let shift = self.traveled();
        let (top, tall) = self.body();
        let (left, right) = (RUNNER_X - 13.0, RUNNER_X + 13.0);

        for hurdle in &self.hurdles {
            let at = hurdle.start - shift;
            let overlap = at < right && at + WIDE > left;
            if !overlap {
                continue;
            }
            let (near, far) = match hurdle.kind {
                Kind::Bar => (BAR_TOP, BAR_BOTTOM),
                Kind::Block(span) => (GROUND - span, GROUND),
            };
            if top < far && top + tall > near {
                return Outcome::Lost;
            }
        }

        if self.age >= LASTS { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        frame.rect(0.0, GROUND, width, 4.0, palette::EDGE);
        frame.rect(0.0, GROUND + 4.0, width, frame::HEIGHT as f32 - GROUND - 4.0, palette::PANEL);

        let shift = self.traveled();
        for hurdle in &self.hurdles {
            let at = hurdle.start - shift;
            if at > width || at + WIDE < 0.0 {
                continue;
            }
            match hurdle.kind {
                Kind::Bar => {
                    frame.rect(at, BAR_TOP, WIDE, BAR_BOTTOM - BAR_TOP, palette::BAD);
                    frame.rect(at + 10.0, 0.0, 6.0, BAR_TOP, palette::EDGE);
                }
                Kind::Block(span) => frame.rect(at, GROUND - span, WIDE, span, palette::WARN),
            }
        }

        let finish = course() - shift;
        if finish < width {
            frame.rect(finish, GROUND - 120.0, 5.0, 120.0, palette::GOOD);
            frame.rect(finish, GROUND - 120.0, 40.0, 22.0, palette::GOOD);
        }

        let (top, tall) = self.body();
        frame.rect(RUNNER_X - 13.0, top, 26.0, tall, palette::PLAYER);
    }
}

#[cfg(test)]
#[path = "../tests/games/dash.rs"]
mod tests;
