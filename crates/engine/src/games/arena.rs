//! An icy ring with a live wall, three chasers, and a shove to send each of them into it.

use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::palette;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "arena",
    title: "Aftershock Arena",
    brief: "You are the blue dot in a round arena of ice, and the wall around it is live. \
            Three red dots start north, east, and west of you and chase you wherever you go. \
            Everything slides: you speed up while an arrow is held and coast when it is not. \
            Dots that meet bounce off each other, all of them the same weight. Knock all \
            three red dots into the wall to win. Touch the wall yourself and the round is \
            over.",
    controls: &["arrow keys push you, diagonals too"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Lethal, Tag::Physics, Tag::Overhead, Tag::Slippery],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Arena::new())
}

pub(crate) const MIDDLE: (f32, f32) = (320.0, 240.0);
pub(crate) const RADIUS: f32 = 212.0;
pub(crate) const DOT: f32 = 14.0;
const START: f32 = 150.0;
const PUSH: f32 = 320.0;
/// How hard a chaser can push, and how fast each one wants to go.
const CHASE: f32 = 120.0;
const EAGER: [f32; 3] = [110.0, 130.0, 150.0];
/// The share of speed lost each second to the ice.
const DRAG: f32 = 0.8;
/// Bumps give back a little more than they take.
const BOUNCE: f32 = 1.3;
const WALL: Rgb = Rgb(255, 226, 40);
const ICE: Rgb = Rgb(40, 56, 76);

#[derive(Clone, Copy, Debug)]
pub(crate) struct Body {
    pub(crate) at: (f32, f32),
    pub(crate) pace: (f32, f32),
}

impl Body {
    fn at_rest(angle: f32) -> Body {
        let at = (MIDDLE.0 + libm::cosf(angle) * START, MIDDLE.1 + libm::sinf(angle) * START);
        Body { at, pace: (0.0, 0.0) }
    }

    fn glide(&mut self, push: (f32, f32)) {
        self.pace.0 = (self.pace.0 + push.0 * TICK) * (1.0 - DRAG * TICK);
        self.pace.1 = (self.pace.1 + push.1 * TICK) * (1.0 - DRAG * TICK);
        self.at.0 += self.pace.0 * TICK;
        self.at.1 += self.pace.1 * TICK;
    }

    pub(crate) fn on_wall(&self) -> bool {
        libm::hypotf(self.at.0 - MIDDLE.0, self.at.1 - MIDDLE.1) + DOT >= RADIUS
    }
}

/// Which way a chaser pushes: toward the pace that would take it straight at
/// the player, as hard as it can.
fn chase(body: &Body, target: (f32, f32), eager: f32) -> (f32, f32) {
    let (tx, ty) = (target.0 - body.at.0, target.1 - body.at.1);
    let far = libm::hypotf(tx, ty).max(1.0);
    let (sx, sy) = (tx / far * eager - body.pace.0, ty / far * eager - body.pace.1);
    let steer = libm::hypotf(sx, sy).max(1.0);
    let strength = CHASE.min(steer / TICK);
    (sx / steer * strength, sy / steer * strength)
}

/// Two bodies of the same weight that overlap are pushed apart and, if they
/// are closing, bounce.
fn bump(one: &mut Body, two: &mut Body) {
    walk::bumped((&mut one.at, &mut one.pace), (&mut two.at, &mut two.pace), DOT, BOUNCE);
}

#[derive(Clone)]
pub(crate) struct Arena {
    pub(crate) player: Body,
    /// Each chaser, gone once it has touched the wall.
    pub(crate) chasers: [Option<Body>; 3],
}

impl Arena {
    pub(crate) fn new() -> Self {
        use std::f32::consts::{FRAC_PI_2, PI};
        Arena {
            player: Body::at_rest(FRAC_PI_2),
            chasers: [
                Some(Body::at_rest(-FRAC_PI_2)),
                Some(Body::at_rest(0.0)),
                Some(Body::at_rest(PI)),
            ],
        }
    }
}

impl Game for Arena {
    fn update(&mut self, input: &Input) -> Outcome {
        let (dx, dy) = input.heading().unwrap_or((0.0, 0.0));
        self.player.glide((dx * PUSH, dy * PUSH));

        let target = self.player.at;
        for (chaser, eager) in self.chasers.iter_mut().zip(EAGER) {
            if let Some(body) = chaser {
                body.glide(chase(body, target, eager));
            }
        }

        for one in 0..3 {
            if let Some(body) = &mut self.chasers[one] {
                bump(&mut self.player, body);
            }
            for two in one + 1..3 {
                if let [Some(mut a), Some(mut b)] = [self.chasers[one], self.chasers[two]] {
                    bump(&mut a, &mut b);
                    self.chasers[one] = Some(a);
                    self.chasers[two] = Some(b);
                }
            }
        }

        if self.player.on_wall() {
            return Outcome::Lost;
        }
        for chaser in &mut self.chasers {
            if chaser.is_some_and(|body| body.on_wall()) {
                *chaser = None;
            }
        }
        if self.chasers.iter().all(Option::is_none) { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.disc(MIDDLE.0, MIDDLE.1, RADIUS + 8.0, WALL.mix(palette::BACKDROP, 0.7));
        frame.disc(MIDDLE.0, MIDDLE.1, RADIUS, ICE);
        frame.ring(MIDDLE.0, MIDDLE.1, RADIUS - 70.0, 1.0, ICE.mix(palette::INK, 0.12));
        frame.ring(MIDDLE.0, MIDDLE.1, RADIUS, 5.0, WALL);
        for body in self.chasers.iter().flatten() {
            frame.disc(body.at.0, body.at.1, DOT, palette::BAD);
        }
        frame.disc(self.player.at.0, self.player.at.1, DOT, palette::PLAYER);
    }
}

#[cfg(test)]
#[path = "../tests/games/arena.rs"]
mod tests;
