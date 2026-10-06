//! Three reels of shapes, stopped one at a time, for three alike across a row.

use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "slots",
    title: "Slot Star",
    brief: "Three reels of shapes roll downward, each showing three shapes at a time. Press \
            SPACE to stop them, left to right. A reel caught between two shapes rolls on to \
            the next one and stops there. Once all three are still, the round is won if any \
            of the three rows across holds the same shape three times, and lost if none does. \
            There is only the one spin.",
    controls: &["SPACE stops the leftmost reel still rolling"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Timing],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Slots::new(seed))
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Shape {
    Square,
    Circle,
    Triangle,
    Pentagon,
    Star,
    Diamond,
}

const SHAPES: [Shape; 6] =
    [Shape::Square, Shape::Circle, Shape::Triangle, Shape::Pentagon, Shape::Star, Shape::Diamond];

pub(crate) const REELS: usize = 3;
pub(crate) const ROWS: usize = 3;
/// Every shape twice around each reel.
pub(crate) const LENGTH: usize = 12;
pub(crate) const SLOT: u32 = 90;
/// Pixels a reel rolls in one tick: four shapes a second.
const PACE: u32 = 6;
const WIDE: f32 = 104.0;
const GAP: f32 = 14.0;
const LEFT: f32 = 320.0 - (WIDE * REELS as f32 + GAP * (REELS - 1) as f32) * 0.5;
const TOP: f32 = 92.0;
const STAR: Rgb = Rgb(246, 176, 36);

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Spin {
    Rolling,
    /// Rolling on until it has rolled this far, then still.
    Settling(u32),
    Still,
}

pub(crate) struct Reel {
    pub(crate) strip: [Shape; LENGTH],
    /// How far the reel has rolled, in pixels.
    pub(crate) at: u32,
    pub(crate) spin: Spin,
}

impl Reel {
    /// The shape in a row, counted down from the top, once the reel stands at a
    /// whole slot.
    pub(crate) fn showing(&self, row: usize) -> Shape {
        self.strip[((self.at / SLOT) as i64 - row as i64).rem_euclid(LENGTH as i64) as usize]
    }

    fn roll(&mut self) {
        match self.spin {
            Spin::Rolling => self.at += PACE,
            Spin::Settling(end) => {
                self.at = (self.at + PACE).min(end);
                if self.at == end {
                    self.spin = Spin::Still;
                }
            }
            Spin::Still => {}
        }
    }
}

pub(crate) struct Slots {
    pub(crate) reels: [Reel; REELS],
}

impl Slots {
    pub(crate) fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let reels = std::array::from_fn(|_| {
            let mut strip = [SHAPES[0]; LENGTH];
            for half in strip.as_chunks_mut::<{ SHAPES.len() }>().0 {
                half.copy_from_slice(&SHAPES);
                rng.shuffle(half);
            }
            // Each half holds every shape once, so a shape can only meet itself
            // where the halves join. Swapping the second of the pair with the
            // one after it parts them, and cannot make a new pair, because the
            // one after it already differed from both.
            for join in [SHAPES.len(), 0] {
                if strip[join] == strip[(join + LENGTH - 1) % LENGTH] {
                    strip.swap(join, (join + 1) % LENGTH);
                }
            }
            Reel { strip, at: rng.below(LENGTH * SLOT as usize) as u32, spin: Spin::Rolling }
        });
        Slots { reels }
    }
}

fn shape(frame: &mut Frame, shape: Shape, cx: f32, cy: f32) {
    match shape {
        Shape::Square => frame.rect(cx - 28.0, cy - 28.0, 56.0, 56.0, palette::PLAYER),
        Shape::Circle => frame.disc(cx, cy, 32.0, palette::BAD),
        Shape::Triangle => frame.corners(cx, cy + 6.0, &[38.0; 3], 0.0, palette::GOOD),
        Shape::Pentagon => frame.corners(cx, cy + 2.0, &[34.0; 5], 0.0, palette::MARK),
        Shape::Star => frame.corners(cx, cy + 3.0, &[38.0, 16.0].repeat(5), 0.0, STAR),
        Shape::Diamond => frame.corners(cx, cy, &[36.0, 26.0].repeat(2), 0.0, palette::TEAL),
    }
}

impl Game for Slots {
    fn update(&mut self, input: &Input) -> Outcome {
        if input.pressed(Key::Space)
            && let Some(reel) = self.reels.iter_mut().find(|reel| reel.spin == Spin::Rolling)
        {
            reel.spin = Spin::Settling(reel.at.div_ceil(SLOT) * SLOT);
        }
        self.reels.iter_mut().for_each(Reel::roll);
        if self.reels.iter().any(|reel| reel.spin != Spin::Still) {
            return Outcome::Playing;
        }
        let lined_up = (0..ROWS).any(|row| {
            let first = self.reels[0].showing(row);
            self.reels.iter().all(|reel| reel.showing(row) == first)
        });
        if lined_up { Outcome::Won } else { Outcome::Lost }
    }

    fn draw(&self, frame: &mut Frame) {
        let tall = SLOT as f32 * ROWS as f32;
        for (i, reel) in self.reels.iter().enumerate() {
            let x = LEFT + i as f32 * (WIDE + GAP);
            frame.rect(x, TOP, WIDE, tall, palette::PANEL);
            let (whole, part) = ((reel.at / SLOT) as i64, (reel.at % SLOT) as f32);
            for place in -1..ROWS as i64 {
                let which = reel.strip[(whole - place).rem_euclid(LENGTH as i64) as usize];
                let y = TOP + place as f32 * SLOT as f32 + part + SLOT as f32 * 0.5;
                shape(frame, which, x + WIDE * 0.5, y);
            }
            frame.rect(x, TOP - SLOT as f32, WIDE, SLOT as f32, palette::BACKDROP);
            frame.rect(x, TOP + tall, WIDE, SLOT as f32, palette::BACKDROP);
            frame.outline(x, TOP, WIDE, tall, 2.0, palette::EDGE);
        }
        if let Some(next) = self.reels.iter().position(|reel| reel.spin == Spin::Rolling) {
            let cx = LEFT + next as f32 * (WIDE + GAP) + WIDE * 0.5;
            let y = TOP + tall + 14.0;
            frame.poly(&[[cx, y], [cx + 12.0, y + 14.0], [cx - 12.0, y + 14.0]], palette::INK);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/slots.rs"]
mod tests;
