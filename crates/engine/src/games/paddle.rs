//! Keep a ball bouncing on a paddle for half a minute, steering it by the way the paddle tilts.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "paddle",
    title: "Paddle Power",
    brief: "A ball bounces on the paddle you hold, always to the same height. Move the paddle \
            left and right to stay under it. The paddle tilts the way you are moving, and the \
            ball comes off a tilted paddle a little faster that way, so how you arrive under \
            the ball decides where it goes next. Arriving flat leaves it going the way it \
            already was. The ball bounces off the walls, and starts to the right of the paddle \
            coming across it, so standing still loses it. Keep it up for thirty seconds. Let \
            it fall past the paddle and the round is over.",
    controls: &["LEFT and RIGHT move the paddle, and tilt it the way they move it"],
    pointer: false,
    scoring: Scoring::Boolean,
    varies: false,
    tags: &[Tag::Fixed, Tag::Lethal, Tag::Timing, Tag::Physics],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Paddle::new())
}

pub(crate) const SURVIVE: u32 = 30 * TICK_HZ;
pub(crate) const WALL: f32 = 12.0;
pub(crate) const PADDLE_Y: f32 = 424.0;
pub(crate) const PADDLE_HALF: f32 = 44.0;
const PADDLE_TALL: f32 = 9.0;
const PADDLE_PACE: f32 = 300.0;
/// How far the ends of a tilted paddle sit above and below flat.
const TIP: f32 = 3.0;
pub(crate) const BALL: f32 = 9.0;
/// How fast the ball leaves the paddle, which puts the bounce at a set height.
const BOUNCE: f32 = 800.0;
const PULL: f32 = 1500.0;
/// What a tilted paddle adds to the ball's sideways speed on a bounce, and the
/// most sideways speed a ball can be given.
pub(crate) const NUDGE: f32 = 60.0;
pub(crate) const FASTEST: f32 = 260.0;

const TABLE: Rgb = Rgb(28, 78, 68);
const BLADE: Rgb = Rgb(196, 62, 62);
const STONE: Rgb = Rgb(196, 168, 120);

pub(crate) struct Paddle {
    age: u32,
    pub(crate) paddle: f32,
    /// Which way the paddle tilts: one to the right, minus one to the left, zero for flat.
    tilt: f32,
    pub(crate) ball: (f32, f32),
    pub(crate) speed: (f32, f32),
}

impl Paddle {
    pub(crate) fn new() -> Self {
        Paddle {
            age: 0,
            paddle: frame::WIDTH as f32 * 0.5 - 40.0,
            tilt: 0.0,
            ball: (frame::WIDTH as f32 * 0.5 + 40.0, 60.0),
            speed: (-130.0, 0.0),
        }
    }
}

impl Game for Paddle {
    fn update(&mut self, input: &Input) -> Outcome {
        self.age += 1;
        self.tilt = input.axis(Key::Left, Key::Right);
        self.paddle = (self.paddle + self.tilt * PADDLE_PACE * TICK)
            .clamp(WALL + PADDLE_HALF, frame::WIDTH as f32 - WALL - PADDLE_HALF);

        self.speed.1 += PULL * TICK;
        let was = self.ball.1;
        self.ball.0 += self.speed.0 * TICK;
        self.ball.1 += self.speed.1 * TICK;
        walk::bounced(
            &mut self.ball.0,
            &mut self.speed.0,
            WALL,
            frame::WIDTH as f32 - WALL,
            BALL,
            1.0,
        );
        if self.ball.1 < BALL {
            self.ball.1 = BALL;
            self.speed.1 = self.speed.1.abs();
        }

        // The tick the ball crosses the face of the paddle, not every tick it is
        // below it: a ball that fell past the paddle has gone, and cannot be
        // fetched back up by sliding the paddle under it.
        let face = PADDLE_Y - BALL;
        let over = (self.ball.0 - self.paddle).abs() <= PADDLE_HALF + BALL * 0.5;
        if self.speed.1 > 0.0 && was < face && self.ball.1 >= face && over {
            self.ball.1 = face;
            self.speed.1 = -BOUNCE;
            self.speed.0 = (self.speed.0 + self.tilt * NUDGE).clamp(-FASTEST, FASTEST);
        }
        if self.ball.1 > frame::HEIGHT as f32 + BALL {
            return Outcome::Lost;
        }
        if self.age >= SURVIVE { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let floor = PADDLE_Y + 40.0;
        frame.rect(0.0, 0.0, frame::WIDTH as f32, floor, TABLE);
        frame.rect(0.0, floor, frame::WIDTH as f32, frame::HEIGHT as f32 - floor, palette::PANEL);
        frame.rect(0.0, 0.0, WALL, floor, STONE);
        frame.rect(frame::WIDTH as f32 - WALL, 0.0, WALL, floor, STONE);

        let tip = self.tilt * TIP;
        frame.line(
            self.paddle - PADDLE_HALF,
            PADDLE_Y - tip,
            self.paddle + PADDLE_HALF,
            PADDLE_Y + tip,
            PADDLE_TALL,
            BLADE,
        );
        frame.disc(self.ball.0, self.ball.1, BALL, palette::INK);
    }
}

#[cfg(test)]
#[path = "../tests/games/paddle.rs"]
mod tests;
