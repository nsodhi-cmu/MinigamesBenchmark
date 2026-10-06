//! Slide through six numbered gates in order, each the way its arrow points.

use std::collections::VecDeque;

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::palette;
use crate::rng::Rng;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "ski",
    title: "Slalom Station",
    brief: "You are a skier on ice: a push sets you sliding and you keep sliding long after \
            you let go. Six numbered gates stand about the field, each two posts with an \
            arrow between them. Go through them in order, one to six, the way the arrow \
            points. Going through a gate backwards, out of turn, or a second time ends the \
            round. Nothing tells you which gate is next: keep count yourself.",
    controls: &["hold the arrows to push"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[
        Tag::Bounded,
        Tag::Hostile,
        Tag::Physics,
        Tag::Navigate,
        Tag::Overhead,
        Tag::Memory,
        Tag::Slippery,
    ],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Ski::new(seed))
}

pub(crate) const GATES: usize = 6;
/// Half the opening between a gate's posts.
pub(crate) const HALF: f32 = 34.0;
/// The least distance between two gates, and between a gate and the start.
pub(crate) const GAP: f32 = 120.0;
const INSET: f32 = HALF + 30.0;
const RADIUS: f32 = 8.0;
const ACCEL: f32 = 600.0;
/// The share of speed lost in a second.
const DRAG: f32 = 1.2;
const TRAIL: usize = 24;
/// Ticks a gate just passed is drawn large and red.
const GLOW: u32 = 24;
const BOTTOM: f32 = frame::HEIGHT as f32;
pub(crate) const START: (f32, f32) = (frame::WIDTH as f32 * 0.5, BOTTOM * 0.5);

const SNOW: Rgb = Rgb(228, 236, 244);
const INK: Rgb = Rgb(28, 36, 56);
const POST: Rgb = Rgb(214, 60, 70);

#[derive(Clone, Copy)]
pub(crate) struct Gate {
    pub(crate) at: (f32, f32),
    /// The way through, as a unit vector.
    pub(crate) way: (f32, f32),
}

impl Gate {
    pub(crate) fn across(self) -> (f32, f32) {
        (-self.way.1, self.way.0)
    }

    /// Whether a move from `from` to `to` passes between the posts, and if so
    /// whether it goes the way the arrow points.
    pub(crate) fn crossed(self, from: (f32, f32), to: (f32, f32)) -> Option<bool> {
        let along = |p: (f32, f32)| (p.0 - self.at.0) * self.way.0 + (p.1 - self.at.1) * self.way.1;
        let (before, after) = (along(from), along(to));
        let forward = before < 0.0 && after >= 0.0;
        if !forward && !(before > 0.0 && after <= 0.0) {
            return None;
        }
        let t = before / (before - after);
        let hit = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
        let across = self.across();
        let lateral = (hit.0 - self.at.0) * across.0 + (hit.1 - self.at.1) * across.1;
        (lateral.abs() <= HALF).then_some(forward)
    }
}

/// Six gates dealt anywhere on the field that is at least `GAP` from the start
/// and from every gate already placed, each facing a random way.
pub(crate) fn lay_out(rng: &mut Rng) -> Vec<Gate> {
    let mut gates: Vec<Gate> = Vec::new();
    while gates.len() < GATES {
        let at = (rng.range(INSET, frame::WIDTH as f32 - INSET), rng.range(INSET, BOTTOM - INSET));
        let far = |other: (f32, f32)| libm::hypotf(at.0 - other.0, at.1 - other.1) >= GAP;
        if far(START) && gates.iter().all(|gate| far(gate.at)) {
            let angle = rng.range(0.0, std::f32::consts::TAU);
            gates.push(Gate { at, way: (libm::cosf(angle), libm::sinf(angle)) });
        }
    }
    gates
}

pub(crate) struct Ski {
    pub(crate) gates: Vec<Gate>,
    pub(crate) at: (f32, f32),
    pub(crate) pace: (f32, f32),
    pub(crate) passed: usize,
    /// Ticks left of the last gate's glow.
    glow: u32,
    trail: VecDeque<(f32, f32)>,
    settled: Option<Outcome>,
}

impl Ski {
    pub(crate) fn new(seed: u64) -> Self {
        let gates = lay_out(&mut Rng::new(seed));
        Ski {
            gates,
            at: START,
            pace: (0.0, 0.0),
            passed: 0,
            glow: 0,
            trail: VecDeque::new(),
            settled: None,
        }
    }
}

impl Game for Ski {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some(settled) = self.settled {
            return settled;
        }
        if let Some((x, y)) = input.heading() {
            self.pace = (self.pace.0 + x * ACCEL * TICK, self.pace.1 + y * ACCEL * TICK);
        }
        self.pace = (self.pace.0 * (1.0 - DRAG * TICK), self.pace.1 * (1.0 - DRAG * TICK));
        let from = self.at;
        self.at = (from.0 + self.pace.0 * TICK, from.1 + self.pace.1 * TICK);
        walk::bounced(&mut self.at.0, &mut self.pace.0, 0.0, frame::WIDTH as f32, RADIUS, 0.5);
        walk::bounced(&mut self.at.1, &mut self.pace.1, 0.0, BOTTOM, RADIUS, 0.5);
        self.trail.push_back(from);
        if self.trail.len() > TRAIL {
            self.trail.pop_front();
        }
        self.glow = self.glow.saturating_sub(1);

        let outcome = match self.gates.iter().enumerate().find_map(|(which, gate)| {
            gate.crossed(from, self.at).map(|forward| forward && which == self.passed)
        }) {
            None => return Outcome::Playing,
            Some(false) => Outcome::Lost,
            Some(true) => {
                self.passed += 1;
                self.glow = GLOW;
                if self.passed == GATES { Outcome::Won } else { Outcome::Playing }
            }
        };
        if outcome != Outcome::Playing {
            self.settled = Some(outcome);
        }
        outcome
    }

    fn draw(&self, frame: &mut Frame) {
        frame.rect(0.0, 0.0, frame::WIDTH as f32, BOTTOM, SNOW);
        for (age, &(x, y)) in self.trail.iter().enumerate() {
            frame.disc(x, y, 2.0, SNOW.mix(palette::PLAYER, age as f32 / TRAIL as f32 * 0.5));
        }
        for (which, gate) in self.gates.iter().enumerate() {
            let (cx, cy) = gate.at;
            let (ax, ay) = gate.across();
            let (wx, wy) = gate.way;
            frame.line(
                cx - ax * HALF,
                cy - ay * HALF,
                cx + ax * HALF,
                cy + ay * HALF,
                1.0,
                SNOW.mix(INK, 0.3),
            );
            frame.disc(cx - ax * HALF, cy - ay * HALF, 6.0, POST);
            frame.disc(cx + ax * HALF, cy + ay * HALF, 6.0, POST);
            let glowing = self.glow > 0 && which + 1 == self.passed;
            let (grow, tint, scale) = if glowing { (1.5, POST, 3) } else { (1.0, INK, 2) };
            let (tip_x, tip_y) = (cx + wx * 12.0 * grow, cy + wy * 12.0 * grow);
            frame.line(cx - wx * 12.0 * grow, cy - wy * 12.0 * grow, tip_x, tip_y, 3.0, tint);
            for side in [-1.0, 1.0] {
                let (bx, by) = (-wx * 7.0 + ax * 6.0 * side, -wy * 7.0 + ay * 6.0 * side);
                frame.line(tip_x, tip_y, tip_x + bx * grow, tip_y + by * grow, 3.0, tint);
            }
            let label = (which + 1).to_string();
            frame.text_center(
                cx - wx * 32.0,
                cy - wy * 32.0 - frame::text_height(scale) * 0.5,
                &label,
                scale,
                tint,
            );
        }
        frame.disc(self.at.0, self.at.1, RADIUS + 2.0, INK);
        frame.disc(self.at.0, self.at.1, RADIUS, palette::PLAYER);
    }
}

#[cfg(test)]
#[path = "../tests/games/ski.rs"]
mod tests;
