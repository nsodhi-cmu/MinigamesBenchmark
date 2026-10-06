//! Type what is on the screen, exactly.

use crate::frame::{self, Frame, text_width};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::Input;
use crate::palette;
use crate::rng::Rng;
use crate::shell::wrap;
use crate::words::WORDS;

pub const SPEC: Spec = Spec {
    name: "typer",
    title: "Hyper Typer",
    brief: "Twenty five-letter words are written across the middle of the screen. Type \
            them from the beginning, one character at a time, spaces included. A wrong key \
            is counted as a slip and ignored, and the passage only moves on when the right \
            key is struck. A sixth slip ends the round.",
    controls: &["letter keys and SPACE type", "there is nothing to undo"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Language],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Typer::new(seed))
}

const LENGTH: usize = 20;
const SLIPS: u32 = 5;
const COLUMNS: usize = 36;

struct Typer {
    passage: String,
    typed: usize,
    slips: u32,
}

impl Typer {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        Typer { passage: rng.take(WORDS, LENGTH).join(" "), typed: 0, slips: 0 }
    }
}

impl Game for Typer {
    fn update(&mut self, input: &Input) -> Outcome {
        let Some(struck) = input.typed() else {
            return Outcome::Playing;
        };
        let wanted = self.passage.as_bytes()[self.typed] as char;
        if struck == wanted {
            self.typed += 1;
            if self.typed == self.passage.len() {
                return Outcome::Won;
            }
        } else {
            self.slips += 1;
            if self.slips > SLIPS {
                return Outcome::Lost;
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let lines = wrap(&self.passage, COLUMNS);
        let mut seen = 0usize;
        let mut y = 150.0;
        for line in &lines {
            let width = text_width(line, 2);
            let mut x = frame::WIDTH as f32 * 0.5 - width * 0.5;
            for character in line.chars() {
                let tint = if seen < self.typed {
                    palette::GOOD
                } else if seen == self.typed {
                    frame.rect(x, y - 4.0, 16.0, 24.0, palette::PANEL);
                    palette::WARN
                } else {
                    palette::MUTED
                };
                frame.text(x, y, &character.to_string(), 2, tint);
                x += 16.0;
                seen += 1;
            }
            // The space a line break swallowed still has to be typed, so the
            // cursor waits at the end of the line rather than nowhere at all.
            if seen == self.typed {
                frame.rect(x, y - 4.0, 16.0, 24.0, palette::PANEL);
            }
            seen += 1;
            y += 40.0;
        }
        let tint = if self.slips == SLIPS { palette::BAD } else { palette::MUTED };
        frame.text_center(
            frame::WIDTH as f32 * 0.5,
            390.0,
            &format!("{} of {} slips used", self.slips, SLIPS),
            1,
            tint,
        );
    }
}
