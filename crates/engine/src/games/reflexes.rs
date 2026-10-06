//! Wait for the color to change, then click as fast as you can.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK_HZ, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "reflexes",
    title: "Real Reflexes",
    brief: "The panel sits red for a few seconds and then turns green. Click as soon as it \
            does: you have half a second, which is thirty frames. How long you took is put \
            up after each one. Click before it turns and the round is over, and so it is if \
            you are too slow. Five in a row finishes it.",
    controls: &["left click when the panel turns green"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Lethal, Tag::Timing],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Reflexes::new(seed))
}

const ROUNDS: u32 = 5;
/// Half a second to answer, counted in frames of game time.
const WINDOW: u32 = 30;
const SOONEST: u32 = 3 * TICK_HZ;
const LATEST: u32 = 5 * TICK_HZ;
const REVEAL: u32 = 78;
const PANEL_TOP: f32 = 80.0;
const PANEL_TALL: f32 = 260.0;
const PANEL_WIDE: f32 = 460.0;

enum Phase {
    Waiting { left: u32 },
    Flashed { since: u32 },
    Showing { frames: u32, left: u32 },
}

struct Reflexes {
    rng: Rng,
    phase: Phase,
    passed: u32,
}

impl Reflexes {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let left = Self::hold(&mut rng);
        Reflexes { rng, phase: Phase::Waiting { left }, passed: 0 }
    }

    /// Somewhere from three to five seconds, to the nearest frame.
    fn hold(rng: &mut Rng) -> u32 {
        SOONEST + rng.below((LATEST - SOONEST + 1) as usize) as u32
    }
}

impl Game for Reflexes {
    fn update(&mut self, input: &Input) -> Outcome {
        let struck = input.button_pressed(Button::Left);
        match self.phase {
            Phase::Waiting { left } => {
                if struck {
                    return Outcome::Lost;
                }
                self.phase = if left <= 1 {
                    Phase::Flashed { since: 0 }
                } else {
                    Phase::Waiting { left: left - 1 }
                };
            }
            Phase::Flashed { since } => {
                if struck {
                    self.phase = Phase::Showing { frames: since, left: REVEAL };
                } else if since + 1 >= WINDOW {
                    return Outcome::Lost;
                } else {
                    self.phase = Phase::Flashed { since: since + 1 };
                }
            }
            Phase::Showing { frames, left } => {
                if left > 1 {
                    self.phase = Phase::Showing { frames, left: left - 1 };
                } else {
                    self.passed += 1;
                    if self.passed == ROUNDS {
                        return Outcome::Won;
                    }
                    let hold = Self::hold(&mut self.rng);
                    self.phase = Phase::Waiting { left: hold };
                }
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let middle = frame::WIDTH as f32 * 0.5;
        let left = middle - PANEL_WIDE * 0.5;
        let (fill, word) = match self.phase {
            Phase::Waiting { .. } => (palette::BAD, "WAIT"),
            Phase::Flashed { .. } => (palette::GOOD, "CLICK"),
            Phase::Showing { .. } => (palette::PANEL, ""),
        };
        frame.bordered(left, PANEL_TOP, PANEL_WIDE, PANEL_TALL, 3.0, fill, palette::EDGE);

        match self.phase {
            Phase::Showing { frames, .. } => {
                frame.text_center(middle, PANEL_TOP + 84.0, &frames.to_string(), 7, palette::INK);
                frame.text_center(middle, PANEL_TOP + 168.0, "FRAMES", 2, palette::MUTED);
            }
            _ => {
                frame.text_center(middle, PANEL_TOP + 116.0, word, 5, palette::BACKDROP);
            }
        }

        frame.pips(middle, 420.0, ROUNDS, self.passed, palette::GOOD);
    }
}

#[cfg(test)]
#[path = "../tests/games/reflexes.rs"]
mod tests;
