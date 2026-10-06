//! Blow a balloon up to a dashed line and send it off.

use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "balloon",
    title: "Balloon Blastoff",
    brief: "A small balloon sits inside a faint dashed circle. Every press of SPACE \
            blows it up a little, and left alone it slowly goes down. Blow it up until its \
            edge is on the dashed circle, then press UP to launch it. Launched near enough \
            the circle, it flies and you win; launched too small or too big, it does not and \
            the round is over. Blow it far past the circle and it bursts.",
    controls: &["tap SPACE to blow it up", "UP launches it"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Strict, Tag::Timing],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Balloon::new())
}

const MIDDLE: (f32, f32) = (320.0, 209.0);
pub(crate) const START: f32 = 24.0;
pub(crate) const TARGET: f32 = 165.0;
/// How far from the dashed circle a launch still flies.
pub(crate) const NEAR: f32 = 7.0;
pub(crate) const BURST: f32 = 175.0;
pub(crate) const PUFF: f32 = 6.0;
/// Pixels of radius lost a second when nothing is pressed.
const LEAK: f32 = 8.0;
const SKIN: Rgb = Rgb(236, 92, 110);

pub(crate) struct Balloon {
    pub(crate) radius: f32,
}

impl Balloon {
    pub(crate) fn new() -> Self {
        Balloon { radius: START }
    }
}

impl Game for Balloon {
    fn update(&mut self, input: &Input) -> Outcome {
        if input.pressed(Key::Up) {
            return if (self.radius - TARGET).abs() <= NEAR { Outcome::Won } else { Outcome::Lost };
        }
        if input.pressed(Key::Space) {
            self.radius += PUFF;
        } else {
            self.radius = (self.radius - LEAK * TICK).max(START);
        }
        if self.radius >= BURST { Outcome::Lost } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let (x, y) = MIDDLE;
        let bottom = y + self.radius;
        frame.line(x, bottom + 8.0, x, 453.0, 2.0, palette::MUTED);
        frame.poly(&[[x, bottom - 2.0], [x + 8.0, bottom + 10.0], [x - 8.0, bottom + 10.0]], SKIN);
        frame.disc(x, y, self.radius, SKIN);
        frame.disc(
            x - self.radius * 0.35,
            y - self.radius * 0.4,
            self.radius * 0.14,
            SKIN.mix(palette::INK, 0.5),
        );

        let dashes = 48;
        let turn = |step: usize| step as f32 / dashes as f32 * std::f32::consts::TAU;
        let tint = palette::MUTED.mix(palette::BACKDROP, 0.35);
        for dash in (0..dashes).step_by(2) {
            let (a, b) = (turn(dash), turn(dash + 1));
            frame.line(
                x + libm::cosf(a) * TARGET,
                y + libm::sinf(a) * TARGET,
                x + libm::cosf(b) * TARGET,
                y + libm::sinf(b) * TARGET,
                2.0,
                tint,
            );
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/balloon.rs"]
mod tests;
