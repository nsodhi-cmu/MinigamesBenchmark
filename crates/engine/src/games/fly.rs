//! Tap your way through the gaps for thirty seconds.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "fly",
    title: "High Flyer",
    brief: "Tap SPACE to give the craft a lift; it sinks the rest of the time. The roof and \
            the floor are solid and will not hurt you, but touch one of the pillars and the \
            round is over. Twenty of them stand between you and the thirty seconds it takes \
            to finish.",
    controls: &["tap SPACE to lift", "each tap is one lift, so let go between them"],
    pointer: false,
    scoring: Scoring::Boolean,
    varies: true,
    tags: &[Tag::Fixed, Tag::Lethal, Tag::Timing, Tag::Physics],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Fly::new(seed))
}

const LASTS: u32 = 30 * TICK_HZ;
pub(crate) const PILLARS: usize = 20;
const ROOF: f32 = 42.0;
const FLOOR: f32 = 440.0;
const CRAFT_X: f32 = 112.0;
const CRAFT: f32 = 11.0;
const SCROLL: f32 = 300.0;
/// A lift is gentle and often, rather than one hard flap and a long fall.
const LIFT: f32 = 330.0;
const WEIGHT: f32 = 900.0;
const FASTEST_FALL: f32 = 540.0;
const WIDE: f32 = 46.0;
pub(crate) const GAP: f32 = 150.0;
const OPENING: f32 = 300.0;
const CLOSING: f32 = 100.0;
/// How far a gap may sit from the one before it, kept inside what a climb covers.
pub(crate) const STEP: f32 = 118.0;

pub(crate) fn gaps_of(seed: u64) -> Vec<(f32, f32)> {
    let mut rng = Rng::new(seed);
    let high = ROOF + GAP * 0.5 + 26.0;
    let low = FLOOR - GAP * 0.5 - 26.0;
    let first = frame::WIDTH as f32 + OPENING;
    let last = CRAFT_X + SCROLL * (LASTS as f32 * TICK) - SCROLL * 1.6 - CLOSING;
    let spread = (last - first) / (PILLARS - 1) as f32;
    let mut middle = (high + low) * 0.5;
    (0..PILLARS)
        .map(|slot| {
            middle = (middle + rng.range(-STEP, STEP)).clamp(high, low);
            (first + slot as f32 * spread + rng.range(-64.0, 64.0), middle)
        })
        .collect()
}

struct Fly {
    gaps: Vec<(f32, f32)>,
    age: u32,
    at: f32,
    fall: f32,
}

impl Fly {
    fn new(seed: u64) -> Self {
        Fly { gaps: gaps_of(seed), age: 0, at: (ROOF + FLOOR) * 0.5, fall: 0.0 }
    }

    fn traveled(&self) -> f32 {
        SCROLL * (self.age as f32 * TICK)
    }
}

impl Game for Fly {
    fn update(&mut self, input: &Input) -> Outcome {
        self.age += 1;
        if input.pressed(Key::Space) {
            self.fall = -LIFT;
        }
        self.fall = (self.fall + WEIGHT * TICK).min(FASTEST_FALL);
        self.at += self.fall * TICK;
        if self.at < ROOF + CRAFT {
            self.at = ROOF + CRAFT;
            self.fall = self.fall.max(0.0);
        }
        if self.at > FLOOR - CRAFT {
            self.at = FLOOR - CRAFT;
            self.fall = self.fall.min(0.0);
        }

        let shift = self.traveled();
        for (start, middle) in &self.gaps {
            let x = start - shift;
            if x > CRAFT_X + CRAFT || x + WIDE < CRAFT_X - CRAFT {
                continue;
            }
            let near = CRAFT_X.clamp(x, x + WIDE);
            for (top, tall) in [
                (ROOF, middle - GAP * 0.5 - ROOF),
                (middle + GAP * 0.5, FLOOR - middle - GAP * 0.5),
            ] {
                let up = self.at.clamp(top, top + tall);
                if libm::hypotf(CRAFT_X - near, self.at - up) < CRAFT {
                    return Outcome::Lost;
                }
            }
        }

        if self.age >= LASTS { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        frame.rect(0.0, ROOF - 6.0, width, 6.0, palette::EDGE);
        frame.rect(0.0, FLOOR, width, 6.0, palette::EDGE);
        frame.rect(0.0, FLOOR + 6.0, width, frame::HEIGHT as f32 - FLOOR - 6.0, palette::PANEL);

        let shift = self.traveled();
        for (start, middle) in &self.gaps {
            let x = start - shift;
            if x > width || x + WIDE < 0.0 {
                continue;
            }
            let top = middle - GAP * 0.5;
            frame.rect(x, ROOF, WIDE, top - ROOF, palette::WARN);
            frame.rect(x, top - 8.0, WIDE, 8.0, palette::INK);
            frame.rect(x, middle + GAP * 0.5, WIDE, FLOOR - middle - GAP * 0.5, palette::WARN);
            frame.rect(x, middle + GAP * 0.5, WIDE, 8.0, palette::INK);
        }

        let wall = CRAFT_X + SCROLL * (LASTS as f32 * TICK) - shift;
        if wall < width {
            frame.rect(wall, ROOF, 6.0, FLOOR - ROOF, palette::GOOD);
            frame.rect(wall, ROOF, 44.0, 24.0, palette::GOOD);
        }
        frame.disc(CRAFT_X, self.at, CRAFT, palette::PLAYER);
        frame.rect(CRAFT_X - CRAFT - 4.0, self.at - 2.0, (CRAFT + 4.0) * 2.0, 5.0, palette::INK);
    }
}

#[cfg(test)]
#[path = "../tests/games/fly.rs"]
mod tests;
