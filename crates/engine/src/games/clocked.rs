//! Start a stopwatch you cannot see, and stop it when you think the time is up.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "clocked",
    title: "Clocked In",
    brief: "You are given a length of time. Press SPACE to start counting and SPACE again \
            when you judge that long has passed. Nothing on the screen will help you: there \
            is no clock while you are counting. Come within a tenth of the time asked for \
            and the trial is good. Five good trials finish the round; one bad one ends it.",
    controls: &["SPACE starts the count", "SPACE again stops it"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Timing, Tag::Memory],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Clocked::new(seed))
}

const TRIALS: u32 = 5;
const SHORTEST: u32 = 30;
const LONGEST: u32 = 80;
/// How far off the asked-for time a trial may be, as a part of it.
const MARGIN: f32 = 0.1;
const REVEAL: u32 = 84;

enum Phase {
    Ready,
    Counting { ticks: u32 },
    Showing { ticks: u32, left: u32, good: bool },
}

struct Clocked {
    asked: u32,
    /// The lengths still to be asked for, last first.
    coming: Vec<u32>,
    phase: Phase,
    passed: u32,
}

impl Clocked {
    fn new(seed: u64) -> Self {
        let mut coming = Self::spread(&mut Rng::new(seed));
        let asked = coming.pop().expect("there is a trial to ask for");
        Clocked { asked, coming, phase: Phase::Ready, passed: 0 }
    }

    /// One length of time from each of `TRIALS` equal parts of the range, in a
    /// random order, so that no round asks for five of a kind. Each is a round
    /// tenth of a second.
    fn spread(rng: &mut Rng) -> Vec<u32> {
        let span = LONGEST - SHORTEST + 1;
        let mut lengths: Vec<u32> = (0..TRIALS)
            .map(|part| {
                let low = SHORTEST + part * span / TRIALS;
                let high = SHORTEST + (part + 1) * span / TRIALS;
                (low + rng.below((high - low) as usize) as u32) * (TICK_HZ / 10)
            })
            .collect();
        rng.shuffle(&mut lengths);
        lengths
    }
}

fn spell(ticks: u32) -> String {
    format!("{:.1}s", ticks as f32 * TICK)
}

impl Game for Clocked {
    fn update(&mut self, input: &Input) -> Outcome {
        let struck = input.pressed(Key::Space);
        match self.phase {
            Phase::Ready => {
                if struck {
                    self.phase = Phase::Counting { ticks: 0 };
                }
            }
            Phase::Counting { ticks } => {
                if struck {
                    let slack = (self.asked as f32 * MARGIN).round() as u32;
                    let good = ticks.abs_diff(self.asked) <= slack;
                    self.phase = Phase::Showing { ticks, left: REVEAL, good };
                } else {
                    self.phase = Phase::Counting { ticks: ticks + 1 };
                }
            }
            Phase::Showing { ticks, left, good } => {
                if left > 1 {
                    self.phase = Phase::Showing { ticks, left: left - 1, good };
                } else if !good {
                    return Outcome::Lost;
                } else {
                    self.passed += 1;
                    if self.passed == TRIALS {
                        return Outcome::Won;
                    }
                    self.asked = self.coming.pop().expect("a trial is left to ask for");
                    self.phase = Phase::Ready;
                }
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let middle = frame::WIDTH as f32 * 0.5;
        frame.text_center(middle, 40.0, "COUNT OUT", 2, palette::MUTED);
        frame.text_center(middle, 76.0, &spell(self.asked), 6, palette::INK);

        match self.phase {
            Phase::Ready => {
                frame.text_center(middle, 260.0, "PRESS SPACE TO START", 2, palette::WARN);
            }
            Phase::Counting { .. } => {
                frame.disc(middle, 268.0, 34.0, palette::GOOD);
                frame.text_center(middle, 322.0, "PRESS SPACE TO STOP", 2, palette::MUTED);
            }
            Phase::Showing { ticks, good, .. } => {
                let tint = if good { palette::GOOD } else { palette::BAD };
                frame.text_center(middle, 250.0, &spell(ticks), 4, tint);
                let word = if good { "GOOD" } else { "OUT BY" };
                frame.text_center(middle, 300.0, word, 2, tint);
                if !good {
                    frame.text_center(middle, 330.0, &spell(ticks.abs_diff(self.asked)), 2, tint);
                }
            }
        }

        frame.pips(middle, 430.0, TRIALS, self.passed, palette::GOOD);
    }
}

#[cfg(test)]
#[path = "../tests/games/clocked.rs"]
mod tests;
