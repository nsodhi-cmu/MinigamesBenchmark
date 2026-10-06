//! Swipe through what is thrown up, and leave the marked ones alone.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;
use crate::shape;

pub const SPEC: Spec = Spec {
    name: "cleaver",
    title: "Clever Cleaver",
    brief: "Forty blocks are thrown up from below over thirty seconds: twenty pale ones to \
            cut and twenty marked with a cross to leave alone. Hold the left button and \
            sweep the pointer through a pale block to cut it. Resting the pointer on one \
            does nothing; the stroke has to be moving. Cut a marked block, or let a single \
            pale one fall away, and the round is over.",
    controls: &["hold the left button and sweep the pointer to cut", "release to stop cutting"],
    pointer: true,
    scoring: Scoring::Boolean,
    varies: true,
    tags: &[Tag::Fixed, Tag::Lethal, Tag::Aim, Tag::Drag],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Cleaver::new(seed))
}

const LASTS: u32 = 30 * TICK_HZ;
const PALE: usize = 20;
const TOSSES: usize = PALE * 2;
const FIRST: f32 = 0.7;
const LAST: f32 = 27.0;
const SIDE: f32 = 34.0;
const GRAVITY: f32 = 540.0;
const FLOOR: f32 = frame::HEIGHT as f32 + SIDE;
/// How far the pointer must travel in a tick for the stroke to cut anything.
const STROKE: f32 = 4.0;

struct Toss {
    tick: u32,
    at: f32,
    drift: (f32, f32),
    marked: bool,
}

struct Block {
    at: (f32, f32),
    drift: (f32, f32),
    marked: bool,
}

struct Cleaver {
    tosses: Vec<Toss>,
    thrown: usize,
    blocks: Vec<Block>,
    age: u32,
    last: (f32, f32),
}

impl Cleaver {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut marks = [false; TOSSES];
        for (slot, mark) in marks.iter_mut().enumerate() {
            *mark = slot >= PALE;
        }
        rng.shuffle(&mut marks);

        let spread = (LAST - FIRST) / (TOSSES - 1) as f32;
        let tosses = marks
            .iter()
            .enumerate()
            .map(|(slot, marked)| {
                let when = FIRST + slot as f32 * spread + rng.range(-0.22, 0.22);
                let at = rng.range(90.0, frame::WIDTH as f32 - 90.0);
                let aim = (frame::WIDTH as f32 * 0.5 - at) * 0.16;
                Toss {
                    tick: (when.max(0.2) * TICK_HZ as f32) as u32,
                    at,
                    drift: (aim, -rng.range(590.0, 690.0)),
                    marked: *marked,
                }
            })
            .collect();

        Cleaver {
            tosses,
            thrown: 0,
            blocks: Vec::new(),
            age: 0,
            last: (frame::WIDTH as f32 * 0.5, frame::HEIGHT as f32 * 0.5),
        }
    }
}

/// Whether the stroke from `a` to `b` touches the square centered on `at`.
fn cuts(a: (f32, f32), b: (f32, f32), at: (f32, f32)) -> bool {
    shape::stroke_hits(a, b, at, (SIDE * 0.5, SIDE * 0.5))
}

impl Game for Cleaver {
    fn update(&mut self, input: &Input) -> Outcome {
        self.age += 1;
        while self.thrown < self.tosses.len() && self.tosses[self.thrown].tick <= self.age {
            let toss = &self.tosses[self.thrown];
            self.blocks.push(Block {
                at: (toss.at, FLOOR),
                drift: toss.drift,
                marked: toss.marked,
            });
            self.thrown += 1;
        }

        let now = input.cursor();
        let travel = ((now.0 - self.last.0).powi(2) + (now.1 - self.last.1).powi(2)).sqrt();
        let holding = input.button_held(Button::Left) || input.button_released(Button::Left);
        let sweeping = holding && travel >= STROKE;

        let mut index = 0;
        while index < self.blocks.len() {
            let block = &mut self.blocks[index];
            block.drift.1 += GRAVITY * TICK;
            block.at.0 += block.drift.0 * TICK;
            block.at.1 += block.drift.1 * TICK;

            let struck = sweeping && cuts(self.last, now, block.at);
            let gone = block.at.1 > FLOOR && block.drift.1 > 0.0;
            let marked = block.marked;
            if struck {
                if marked {
                    return Outcome::Lost;
                }
                self.blocks.swap_remove(index);
                continue;
            }
            if gone {
                if !marked {
                    return Outcome::Lost;
                }
                self.blocks.swap_remove(index);
                continue;
            }
            index += 1;
        }

        self.last = now;
        if self.age >= LASTS { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        for block in &self.blocks {
            let (x, y) = (block.at.0 - SIDE * 0.5, block.at.1 - SIDE * 0.5);
            if block.marked {
                frame.bordered(x, y, SIDE, SIDE, 2.0, palette::PANEL, palette::BAD);
                frame.cross(x + SIDE * 0.5, y + SIDE * 0.5, SIDE * 0.5 - 7.0, 3.0, palette::BAD);
            } else {
                frame.bordered(x, y, SIDE, SIDE, 6.0, palette::MARK, palette::INK);
            }
        }
    }
}
