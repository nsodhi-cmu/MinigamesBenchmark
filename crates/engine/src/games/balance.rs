//! Tip a beam until the ball sits still where you want it.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "balance",
    title: "Balance Ball",
    brief: "A ball rolls on a beam you can tip left and right. Bring the ball inside the \
            green band and keep it there for three unbroken seconds. Roll the ball off \
            either end of the beam and the round is over.",
    controls: &["hold LEFT to tip the beam left", "hold RIGHT to tip it right"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Hostile, Tag::Physics],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Balance::new())
}

const PIVOT: (f32, f32) = (320.0, 300.0);
const REACH: f32 = 224.0;
const THICK: f32 = 10.0;
const BALL: f32 = 13.0;
const TILT_MAX: f32 = 0.34;
const TILT_RATE: f32 = 1.15;
const PULL: f32 = 780.0;
const DRAG: f32 = 0.4;
/// Well right of the middle, and well short of the end.
const ZONE_AT: f32 = 130.0;
const ZONE_HALF: f32 = 30.0;
const HOLD: u32 = 3 * TICK_HZ;

struct Balance {
    tilt: f32,
    along: f32,
    speed: f32,
    held: u32,
}

impl Balance {
    fn new() -> Self {
        Balance { tilt: 0.0, along: 0.0, speed: 0.0, held: 0 }
    }

    fn axes(&self) -> ((f32, f32), (f32, f32)) {
        let (sin, cos) = libm::sincosf(self.tilt);
        ((cos, sin), (-sin, cos))
    }

    fn ball(&self) -> (f32, f32) {
        let (along, across) = self.axes();
        let lift = THICK * 0.5 + BALL;
        (
            PIVOT.0 + along.0 * self.along - across.0 * lift,
            PIVOT.1 + along.1 * self.along - across.1 * lift,
        )
    }

    fn on_beam(&self, offset: f32, out: f32) -> [f32; 2] {
        let (along, across) = self.axes();
        [PIVOT.0 + along.0 * offset + across.0 * out, PIVOT.1 + along.1 * offset + across.1 * out]
    }
}

impl Game for Balance {
    fn update(&mut self, input: &Input) -> Outcome {
        let lean = input.axis(Key::Left, Key::Right);
        self.tilt = (self.tilt + lean * TILT_RATE * TICK).clamp(-TILT_MAX, TILT_MAX);

        self.speed += (PULL * libm::sinf(self.tilt) - DRAG * self.speed) * TICK;
        self.along += self.speed * TICK;

        if self.along.abs() > REACH {
            return Outcome::Lost;
        }

        if (self.along - ZONE_AT).abs() <= ZONE_HALF {
            self.held += 1;
        } else {
            self.held = 0;
        }
        if self.held >= HOLD { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let half = THICK * 0.5;
        frame.poly(
            &[
                self.on_beam(-REACH, -half),
                self.on_beam(REACH, -half),
                self.on_beam(REACH, half),
                self.on_beam(-REACH, half),
            ],
            palette::EDGE,
        );
        frame.poly(
            &[
                self.on_beam(ZONE_AT - ZONE_HALF, -half),
                self.on_beam(ZONE_AT + ZONE_HALF, -half),
                self.on_beam(ZONE_AT + ZONE_HALF, half),
                self.on_beam(ZONE_AT - ZONE_HALF, half),
            ],
            palette::GOOD,
        );
        frame.poly(
            &[
                [PIVOT.0 - 18.0, PIVOT.1 + 62.0],
                [PIVOT.0 + 18.0, PIVOT.1 + 62.0],
                [PIVOT.0, PIVOT.1],
            ],
            palette::PANEL,
        );

        let (x, y) = self.ball();
        frame.disc(x, y, BALL, palette::PLAYER);

        let width = 240.0;
        let left = frame::WIDTH as f32 * 0.5 - width * 0.5;
        frame.outline(left, 92.0, width, 18.0, 2.0, palette::EDGE);
        frame.rect(
            left + 2.0,
            94.0,
            (width - 4.0) * (self.held as f32 / HOLD as f32),
            14.0,
            palette::GOOD,
        );
    }
}
