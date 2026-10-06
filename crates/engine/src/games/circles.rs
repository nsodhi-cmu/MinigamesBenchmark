//! Click the shrinking ring before it closes.

use crate::frame::Frame;
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "circles",
    title: "Clicking Circles",
    brief: "A ring appears somewhere on the field and closes in on its center. Put the \
            pointer inside the ring and click before it disappears. Twenty rings finish \
            the round. Let one of them close on you and the round is over.",
    controls: &["move the pointer", "left click"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Lethal, Tag::Aim, Tag::Timing],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Bullseye::new(seed))
}

const TARGETS: u32 = 20;
const LIFE: u32 = 96;
const WIDE: f32 = 46.0;
const TIGHT: f32 = 9.0;
const MARGIN: f32 = 56.0;

struct Bullseye {
    rng: Rng,
    at: (f32, f32),
    age: u32,
    hits: u32,
}

impl Bullseye {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let at = place(&mut rng);
        Bullseye { rng, at, age: 0, hits: 0 }
    }

    fn radius(&self) -> f32 {
        WIDE + (TIGHT - WIDE) * (self.age as f32 / LIFE as f32)
    }

    fn respawn(&mut self) {
        self.at = place(&mut self.rng);
        self.age = 0;
    }
}

fn place(rng: &mut Rng) -> (f32, f32) {
    (
        rng.range(MARGIN, crate::frame::WIDTH as f32 - MARGIN),
        rng.range(MARGIN, crate::frame::HEIGHT as f32 - MARGIN),
    )
}

impl Game for Bullseye {
    fn update(&mut self, input: &Input) -> Outcome {
        self.age += 1;
        if input.button_pressed(Button::Left) {
            let (x, y) = input.cursor();
            let (dx, dy) = (x - self.at.0, y - self.at.1);
            let r = self.radius();
            if dx * dx + dy * dy <= r * r {
                self.hits += 1;
                if self.hits == TARGETS {
                    return Outcome::Won;
                }
                self.respawn();
            }
        }
        if self.age >= LIFE {
            return Outcome::Lost;
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let (x, y) = self.at;
        let r = self.radius();
        frame.ring(x, y, WIDE, 1.0, palette::PINK.mix(palette::BACKDROP, 0.6));
        frame.disc(x, y, r, palette::PINK);
        frame.disc(x, y, r * 0.35, palette::PINK.mix(palette::BACKDROP, 0.45));
    }
}
