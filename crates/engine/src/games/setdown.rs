//! Bring a falling craft down on the pad, gently and in the right place.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "setdown",
    title: "Soft Setdown",
    brief: "A craft falls under its own weight. Hold SPACE or UP to fire the thruster under it \
            and the arrow keys to lean it left or right. Put it down on the green pad \
            slowly enough that the bar on the left is still green. Land it hard, or miss \
            the pad, and the round is over. Five landings finish it.",
    controls: &["hold SPACE or UP to thrust", "LEFT and RIGHT lean"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Lethal, Tag::Physics],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Setdown::new(seed))
}

const LANDINGS: u32 = 5;
const GROUND: f32 = 430.0;
const CRAFT: f32 = 20.0;
const HALF: f32 = CRAFT * 0.5;
const PAD: f32 = 92.0;
const WEIGHT: f32 = 250.0;
const THRUST: f32 = 620.0;
const LEAN: f32 = 240.0;
/// The rate at which sideways speed dies away, per second.
const DRAG: f32 = 1.7;
const DRIFT_LIMIT: f32 = 62.0;
const DROP_LIMIT: f32 = 96.0;

struct Setdown {
    rng: Rng,
    at: (f32, f32),
    speed: (f32, f32),
    pad: f32,
    landed: u32,
    firing: bool,
}

impl Setdown {
    fn new(seed: u64) -> Self {
        let mut setdown = Setdown {
            rng: Rng::new(seed),
            at: (0.0, 0.0),
            speed: (0.0, 0.0),
            pad: 0.0,
            landed: 0,
            firing: false,
        };
        setdown.reset();
        setdown
    }

    fn reset(&mut self) {
        self.pad = self.rng.range(PAD, frame::WIDTH as f32 - PAD);
        self.at = (self.rng.range(120.0, frame::WIDTH as f32 - 120.0), 27.0);
        self.speed = (0.0, 0.0);
    }
}

impl Game for Setdown {
    fn update(&mut self, input: &Input) -> Outcome {
        self.firing = input.lift_held();
        let lean = input.axis(Key::Left, Key::Right);
        self.speed.0 += (lean * LEAN - self.speed.0 * DRAG) * TICK;
        self.speed.1 += (WEIGHT - if self.firing { THRUST } else { 0.0 }) * TICK;
        self.at.0 += self.speed.0 * TICK;
        self.at.1 += self.speed.1 * TICK;

        if self.at.0 < HALF || self.at.0 > frame::WIDTH as f32 - HALF {
            self.at.0 = self.at.0.clamp(HALF, frame::WIDTH as f32 - HALF);
            self.speed.0 = 0.0;
        }
        if self.at.1 < HALF {
            self.at.1 = HALF;
            self.speed.1 = self.speed.1.max(0.0);
        }
        if self.at.1 + HALF < GROUND {
            return Outcome::Playing;
        }

        let gentle = self.speed.1 <= DROP_LIMIT && self.speed.0.abs() <= DRIFT_LIMIT;
        let on_pad = (self.at.0 - self.pad).abs() <= PAD * 0.5 - HALF;
        if !gentle || !on_pad {
            return Outcome::Lost;
        }
        self.landed += 1;
        if self.landed == LANDINGS {
            return Outcome::Won;
        }
        self.reset();
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        frame.rect(0.0, GROUND, width, frame::HEIGHT as f32 - GROUND, palette::PANEL);
        frame.rect(0.0, GROUND, width, 3.0, palette::EDGE);
        frame.rect(self.pad - PAD * 0.5, GROUND - 5.0, PAD, 8.0, palette::GOOD);

        let (x, y) = self.at;
        let hot = self.speed.1 > DROP_LIMIT || self.speed.0.abs() > DRIFT_LIMIT;
        frame.rect(x - HALF, y - HALF, CRAFT, CRAFT, palette::PLAYER);
        if hot {
            let ring = CRAFT + 10.0;
            frame.outline(x - ring * 0.5, y - ring * 0.5, ring, ring, 3.0, palette::BAD);
        }
        frame.rect(x - CRAFT * 0.7, y - HALF - 5.0, CRAFT * 1.4, 5.0, palette::INK);
        if self.firing {
            frame.poly(
                &[[x - 7.0, y + HALF], [x + 7.0, y + HALF], [x, y + HALF + 22.0]],
                palette::WARN,
            );
        }

        let (bar_x, bar_y, bar_tall) = (24.0, 27.0, 300.0);
        frame.outline(bar_x, bar_y, 18.0, bar_tall, 2.0, palette::EDGE);
        let safe = bar_tall * (DROP_LIMIT / 260.0);
        frame.rect(
            bar_x + 2.0,
            bar_y + bar_tall - safe,
            14.0,
            safe,
            palette::GOOD.mix(palette::BACKDROP, 0.55),
        );
        let fall = (self.speed.1.max(0.0) / 260.0 * bar_tall).min(bar_tall - 4.0);
        let tint = if self.speed.1 > DROP_LIMIT { palette::BAD } else { palette::GOOD };
        frame.rect(bar_x + 2.0, bar_y + bar_tall - fall - 2.0, 14.0, fall, tint);
        frame.rect(bar_x - 4.0, bar_y + bar_tall - safe, 26.0, 2.0, palette::INK);

        frame.pips(width * 0.5, frame::HEIGHT as f32 - 22.0, LANDINGS, self.landed, palette::GOOD);
    }
}
