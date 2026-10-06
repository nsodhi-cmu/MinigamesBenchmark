//! Pick the word the loose letters spell.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;
use crate::words::WORDS;

pub const SPEC: Spec = Spec {
    name: "cipher",
    title: "Cipher Cinco",
    brief: "Loose letters sit at the top of the screen. One of the four words below uses \
            exactly those letters and no others. Click it. Fifteen right answers finish the \
            round; one wrong one ends it.",
    controls: &["left click a word"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Strict, Tag::Quiz, Tag::Language],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Cipher::new(seed))
}

const ROUNDS: u32 = 15;
const CHOICES: usize = 4;
const BOX_WIDTH: f32 = 220.0;
const BOX_HEIGHT: f32 = 54.0;

struct Cipher {
    rng: Rng,
    scrambled: String,
    choices: [&'static str; CHOICES],
    answer: usize,
    solved: u32,
}

impl Cipher {
    fn new(seed: u64) -> Self {
        let mut cipher = Cipher {
            rng: Rng::new(seed),
            scrambled: String::new(),
            choices: [""; CHOICES],
            answer: 0,
            solved: 0,
        };
        cipher.deal();
        cipher
    }

    fn deal(&mut self) {
        let choices = self.rng.take(WORDS, CHOICES);
        self.answer = self.rng.below(CHOICES);
        let answer = choices[self.answer];
        self.choices = choices.try_into().expect("as many are dealt as are shown");

        // A scramble that came out as the word itself is no puzzle. Swapping
        // the first letter with one that differs from it settles that, and no
        // word here is all one letter.
        let mut letters: Vec<char> = answer.chars().collect();
        self.rng.shuffle(&mut letters);
        if letters.iter().collect::<String>() == answer {
            let first = letters[0];
            let other =
                letters.iter().position(|c| *c != first).expect("a word is not all one letter");
            letters.swap(0, other);
        }
        self.scrambled = letters.iter().map(|c| c.to_ascii_uppercase()).collect();
    }

    fn box_of(slot: usize) -> (f32, f32) {
        let column = (slot % 2) as f32;
        let row = (slot / 2) as f32;
        (
            frame::WIDTH as f32 * 0.5 - BOX_WIDTH - 14.0 + column * (BOX_WIDTH + 28.0),
            250.0 + row * (BOX_HEIGHT + 26.0),
        )
    }
}

impl Game for Cipher {
    fn update(&mut self, input: &Input) -> Outcome {
        if !input.button_pressed(Button::Left) {
            return Outcome::Playing;
        }
        let (x, y) = input.cursor();
        for slot in 0..CHOICES {
            let (bx, by) = Self::box_of(slot);
            if x < bx || y < by || x > bx + BOX_WIDTH || y > by + BOX_HEIGHT {
                continue;
            }
            if slot == self.answer {
                self.solved += 1;
                if self.solved == ROUNDS {
                    return Outcome::Won;
                }
                self.deal();
            } else {
                return Outcome::Lost;
            }
            break;
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let middle = frame::WIDTH as f32 * 0.5;
        let letters: Vec<char> = self.scrambled.chars().collect();
        let span = letters.len() as f32 * 56.0;
        for (index, letter) in letters.iter().enumerate() {
            let x = middle - span * 0.5 + index as f32 * 56.0;
            frame.bordered(x + 4.0, 88.0, 48.0, 60.0, 2.0, palette::PANEL, palette::EDGE);
            frame.text_center(x + 28.0, 102.0, &letter.to_string(), 4, palette::INK);
        }

        for slot in 0..CHOICES {
            let (bx, by) = Self::box_of(slot);
            frame.bordered(bx, by, BOX_WIDTH, BOX_HEIGHT, 2.0, palette::PANEL, palette::MARK);
            frame.text_center(bx + BOX_WIDTH * 0.5, by + 18.0, self.choices[slot], 2, palette::INK);
        }
    }
}
