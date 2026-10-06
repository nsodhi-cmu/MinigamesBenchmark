//! Cross the yard while something is watching it.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "eye",
    title: "Eerie Eye",
    brief: "Hold SPACE to walk right toward the line. The eye above the yard \
            turns away, warns, then watches. Move so much as a step while it is \
            watching and the round is over. Reach the line to finish.",
    controls: &["hold SPACE to walk", "let go to stand still"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Timing],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Sentry::new(seed))
}

const START: f32 = 56.0;
const FINISH: f32 = 592.0;
const LANE: f32 = 356.0;
const PACE: f32 = 55.0;

#[derive(Clone, Copy, PartialEq)]
enum Watch {
    Away,
    Turning,
    Watching,
}

struct Sentry {
    rng: Rng,
    at: f32,
    watch: Watch,
    left: f32,
}

impl Sentry {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let left = rng.range(1.2, 2.4);
        Sentry { rng, at: START, watch: Watch::Away, left }
    }

    fn turn(&mut self) {
        let (next, span) = match self.watch {
            Watch::Away => (Watch::Turning, 0.55),
            Watch::Turning => (Watch::Watching, self.rng.range(1.1, 2.3)),
            Watch::Watching => (Watch::Away, self.rng.range(1.4, 2.9)),
        };
        self.watch = next;
        self.left = span;
    }
}

impl Game for Sentry {
    fn update(&mut self, input: &Input) -> Outcome {
        self.left -= TICK;
        if self.left <= 0.0 {
            self.turn();
        }
        if input.held(Key::Space) {
            if self.watch == Watch::Watching {
                return Outcome::Lost;
            }
            self.at += PACE * TICK;
        }
        if self.at >= FINISH { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        let (tint, word) = match self.watch {
            Watch::Away => (palette::GOOD, "LOOKING AWAY"),
            Watch::Turning => (palette::WARN, "TURNING"),
            Watch::Watching => (palette::BAD, "WATCHING"),
        };
        frame.rect(0.0, 8.0, width, 42.0, tint);
        frame.text_center(width * 0.5, 21.0, word, 2, palette::BACKDROP);

        let (eye_x, eye_y) = (width * 0.5, 190.0);
        frame.disc(eye_x, eye_y, 58.0, palette::PANEL);
        frame.ring(eye_x, eye_y, 58.0, 3.0, tint);
        match self.watch {
            Watch::Away => frame.rect(eye_x - 40.0, eye_y - 3.0, 80.0, 6.0, tint),
            Watch::Turning => {
                frame.disc(eye_x, eye_y, 26.0, palette::EDGE);
                frame.rect(eye_x - 30.0, eye_y - 26.0, 60.0, 26.0, palette::PANEL);
                frame.rect(eye_x - 30.0, eye_y + 12.0, 60.0, 26.0, palette::PANEL);
                frame.disc(eye_x, eye_y, 9.0, tint);
            }
            Watch::Watching => {
                frame.disc(eye_x, eye_y, 34.0, palette::INK);
                frame.disc(eye_x, eye_y, 15.0, tint);
            }
        }

        frame.rect(START - 8.0, LANE - 4.0, FINISH - START + 16.0, 8.0, palette::PANEL);
        frame.rect(FINISH, LANE - 34.0, 6.0, 68.0, palette::GOOD);
        frame.disc(self.at, LANE, 13.0, palette::PLAYER);
    }
}
