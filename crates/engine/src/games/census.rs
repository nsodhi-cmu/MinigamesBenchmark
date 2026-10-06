//! Watch two kinds of thing drift past, then say how many of one of them there were.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "census",
    title: "Con Census",
    brief: "For thirty seconds, blue stars and amber diamonds drift across the screen. You \
            are not told which kind you will be asked to count until the time is up. Then \
            type the number and press ENTER. One answer, right or wrong.",
    controls: &["watch", "then type digits and press ENTER", "BACKSPACE rubs one out"],
    pointer: false,
    scoring: Scoring::Boolean,
    varies: true,
    tags: &[Tag::Fixed, Tag::Strict, Tag::Memory, Tag::Quiz],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Census::new(seed))
}

const WATCH: u32 = 30 * TICK_HZ;
const LAST_ARRIVAL: u32 = 28 * TICK_HZ;
const SIZE: f32 = 15.0;
/// The band a mote stays inside, which is clear of the two shapes named along
/// the bottom, so that everything counted is in sight.
const TOP: f32 = 30.0;
const FLOOR: f32 = 410.0;
const DIGITS: usize = 3;

struct Mote {
    at: (f32, f32),
    drift: (f32, f32),
    star: bool,
}

struct Census {
    rng: Rng,
    motes: Vec<Mote>,
    age: u32,
    wait: f32,
    stars: u32,
    diamonds: u32,
    asked_star: bool,
    answer: String,
    settled: Option<Outcome>,
}

impl Census {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let asked_star = rng.chance(0.5);
        Census {
            rng,
            motes: Vec::new(),
            age: 0,
            wait: 0.4,
            stars: 0,
            diamonds: 0,
            asked_star,
            answer: String::new(),
            settled: None,
        }
    }

    fn arrive(&mut self) {
        let star = self.rng.chance(0.5);
        let leftward = self.rng.chance(0.5);
        let pace = self.rng.range(74.0, 138.0);
        let y = self.rng.range(TOP, FLOOR);
        self.motes.push(Mote {
            at: (if leftward { frame::WIDTH as f32 + SIZE } else { -SIZE }, y),
            drift: (if leftward { -pace } else { pace }, self.rng.range(-26.0, 26.0)),
            star,
        });
        if star {
            self.stars += 1;
        } else {
            self.diamonds += 1;
        }
    }

    fn wanted(&self) -> u32 {
        if self.asked_star { self.stars } else { self.diamonds }
    }
}

fn star(frame: &mut Frame, cx: f32, cy: f32, r: f32, tint: Rgb) {
    frame.corners(cx, cy, &[r, r * 0.4].repeat(4), 0.0, tint);
}

fn diamond(frame: &mut Frame, cx: f32, cy: f32, r: f32, tint: Rgb) {
    frame.corners(cx, cy, &[r, r * 0.74].repeat(2), 0.0, tint);
}

impl Game for Census {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some(settled) = self.settled {
            return settled;
        }
        self.age += 1;

        if self.age < WATCH {
            self.wait -= TICK;
            if self.wait <= 0.0 && self.age < LAST_ARRIVAL {
                self.wait = self.rng.range(0.9, 1.5);
                self.arrive();
            }
            for mote in &mut self.motes {
                mote.at.0 += mote.drift.0 * TICK;
                mote.at.1 += mote.drift.1 * TICK;
                walk::bounced(&mut mote.at.1, &mut mote.drift.1, TOP, FLOOR, 0.0, 1.0);
            }
            self.motes.retain(|mote| {
                mote.at.0 > -SIZE * 2.0 && mote.at.0 < frame::WIDTH as f32 + SIZE * 2.0
            });
            return Outcome::Playing;
        }

        self.motes.clear();
        if let Some(struck) = input.typed()
            && struck.is_ascii_digit()
            && self.answer.len() < DIGITS
        {
            self.answer.push(struck);
        }
        if input.pressed(Key::Backspace) {
            self.answer.pop();
        }
        if input.pressed(Key::Enter) && !self.answer.is_empty() {
            let said: u32 = self.answer.parse().expect("only digits are kept");
            let settled = if said == self.wanted() { Outcome::Won } else { Outcome::Lost };
            self.settled = Some(settled);
            return settled;
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let middle = frame::WIDTH as f32 * 0.5;
        if self.age < WATCH {
            for mote in &self.motes {
                if mote.star {
                    star(frame, mote.at.0, mote.at.1, SIZE, palette::PLAYER);
                } else {
                    diamond(frame, mote.at.0, mote.at.1, SIZE, palette::WARN);
                }
            }
            frame.text_center(middle, 452.0, "COUNT BOTH KINDS", 1, palette::MUTED);
            star(frame, middle - 108.0, 456.0, 13.0, palette::PLAYER);
            diamond(frame, middle + 108.0, 456.0, 13.0, palette::WARN);
            return;
        }

        let (kind, tint) = if self.asked_star {
            ("BLUE STARS", palette::PLAYER)
        } else {
            ("AMBER DIAMONDS", palette::WARN)
        };
        frame.text_center(middle, 120.0, "HOW MANY", 2, palette::MUTED);
        frame.text_center(middle, 158.0, kind, 3, tint);
        if self.asked_star {
            star(frame, middle, 226.0, 20.0, tint);
        } else {
            diamond(frame, middle, 226.0, 20.0, tint);
        }

        frame.bordered(middle - 90.0, 274.0, 180.0, 72.0, 3.0, palette::PANEL, palette::EDGE);
        frame.text_center(middle, 292.0, &self.answer, 4, palette::INK);
        frame.text_center(middle, 380.0, "TYPE THE NUMBER, THEN ENTER", 1, palette::MUTED);
    }
}

#[cfg(test)]
#[path = "../tests/games/census.rs"]
mod tests;
