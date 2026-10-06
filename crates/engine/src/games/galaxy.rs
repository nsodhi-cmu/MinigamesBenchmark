//! Shoot down a formation whose drones take turns diving through you and climbing back.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::sky;

pub const SPEC: Spec = Spec {
    name: "galaxy",
    title: "Galaxy Gunner",
    brief: "Two rows of ten drones hold a formation above you, swaying from side to side. \
            Every few seconds one drone leaves the formation, dives down through the bottom of \
            the screen where you fly, and climbs back to its place. Move along the bottom and fire \
            upward to shoot all twenty down, whether they are in the formation or diving. Two \
            of your shots can be in the air at once. Touching a drone ends the round.",
    controls: &["LEFT and RIGHT move", "SPACE fires"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Lethal, Tag::Timing],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Galaxy::new())
}

const ACROSS: usize = 10;
pub(crate) const DRONES: usize = ACROSS * 2;
const SPACING: f32 = 50.0;
const ROWS: [f32; 2] = [53.0, 97.0];
const DRONE_HALF: f32 = 14.0;
const SWAY: f32 = 50.0;
const SWAY_TICKS: u32 = 4 * TICK_HZ;

pub(crate) const SHIP_Y: f32 = 452.0;
const SHIP_PACE: f32 = 230.0;
const SHOT_PACE: f32 = 600.0;
const SHOTS: usize = 2;

/// How long a dive takes from leaving its place to being back in it.
const DIVE: u32 = 210;
/// How far below the bottom of the ship a dive's lowest point is.
const DEPTH: f32 = 14.0;
const SWING: f32 = 70.0;
/// Ticks from one dive beginning to the next being called.
const CALL: u32 = 150;
/// How far across from its place toward the ship a dive is aimed.
const LEAN_IN: f32 = 0.5;
/// Drones are called in turn, stepping this far along the formation each time.
const STRIDE: usize = 7;

const FRONT: Rgb = palette::PINK;
const BACK: Rgb = Rgb(150, 110, 230);

#[derive(Clone, Copy)]
struct Dive {
    tick: u32,
    /// Where across the bottom the dive was aimed when it began.
    aim: f32,
    swing: f32,
}

#[derive(Clone, Copy)]
enum Drone {
    Down,
    Holding,
    Diving(Dive),
}

#[derive(Clone)]
pub(crate) struct Galaxy {
    age: u32,
    pub(crate) ship: f32,
    drones: [Drone; DRONES],
    pub(crate) shots: Vec<(f32, f32)>,
    call: u32,
    dives: u32,
}

impl Galaxy {
    pub(crate) fn new() -> Self {
        Galaxy {
            age: 0,
            ship: frame::WIDTH as f32 * 0.5,
            drones: [Drone::Holding; DRONES],
            shots: Vec::new(),
            call: CALL,
            dives: 0,
        }
    }

    pub(crate) fn down(&self) -> usize {
        self.drones.iter().filter(|drone| matches!(drone, Drone::Down)).count()
    }

    /// Where a drone's place in the formation is now.
    fn place(&self, which: usize) -> (f32, f32) {
        let span = SPACING * (ACROSS - 1) as f32;
        let phase = (self.age % SWAY_TICKS) as f32 / SWAY_TICKS as f32;
        let sway = SWAY * libm::sinf(phase * std::f32::consts::TAU);
        let x = (frame::WIDTH as f32 - span) * 0.5 + (which % ACROSS) as f32 * SPACING + sway;
        (x, ROWS[which / ACROSS])
    }

    /// Where a drone is, if it is up: down to the bottom and back up again,
    /// out toward where the dive was aimed and back to its place.
    pub(crate) fn position(&self, which: usize) -> Option<(f32, f32)> {
        let (x, y) = self.place(which);
        match self.drones[which] {
            Drone::Down => None,
            Drone::Holding => Some((x, y)),
            Drone::Diving(dive) => {
                let along = (self.age - dive.tick) as f32 / DIVE as f32;
                let arc = libm::sinf(along * std::f32::consts::PI);
                let lean = dive.swing * libm::sinf(along * std::f32::consts::TAU);
                // A dive aimed at a ship parked against a wall would otherwise
                // carry the drone off the picture, where it cannot be shot.
                let across = (x + (dive.aim - x) * arc + lean)
                    .clamp(DRONE_HALF, frame::WIDTH as f32 - DRONE_HALF);
                Some((across, y + (SHIP_Y + DEPTH - y) * arc))
            }
        }
    }

    /// The drone to send next: the next one holding its place, taking the
    /// formation in a fixed order that jumps about it.
    fn caller(&self) -> Option<usize> {
        (0..DRONES)
            .map(|turn| (self.dives as usize + turn) * STRIDE % DRONES)
            .find(|which| matches!(self.drones[*which], Drone::Holding))
    }
}

impl Game for Galaxy {
    fn update(&mut self, input: &Input) -> Outcome {
        self.age += 1;
        let heading = input.axis(Key::Left, Key::Right);
        self.ship = (self.ship + heading * SHIP_PACE * TICK)
            .clamp(sky::SHIP_HALF, frame::WIDTH as f32 - sky::SHIP_HALF);
        if input.pressed(Key::Space) && self.shots.len() < SHOTS {
            self.shots.push((self.ship, SHIP_Y - sky::SHIP_TALL));
        }

        let mut flying = Vec::new();
        for (x, y) in std::mem::take(&mut self.shots) {
            let y = y - SHOT_PACE * TICK;
            let hit = (0..DRONES).find(|which| {
                self.position(*which).is_some_and(|(dx, dy)| {
                    (dx - x).abs() <= DRONE_HALF && (dy - y).abs() <= DRONE_HALF
                })
            });
            match hit {
                Some(which) => self.drones[which] = Drone::Down,
                None if y > 0.0 => flying.push((x, y)),
                None => {}
            }
        }
        self.shots = flying;
        if self.down() == DRONES {
            return Outcome::Won;
        }

        for which in 0..DRONES {
            if let Drone::Diving(dive) = self.drones[which]
                && self.age - dive.tick >= DIVE
            {
                self.drones[which] = Drone::Holding;
            }
        }
        let diving = self.drones.iter().filter(|drone| matches!(drone, Drone::Diving(_))).count();
        self.call = self.call.saturating_sub(1);
        if self.call == 0 && diving == 0 {
            if let Some(which) = self.caller() {
                let place = self.place(which).0;
                let aim = place + (self.ship - place) * LEAN_IN;
                let swing = if self.dives.is_multiple_of(2) { SWING } else { -SWING };
                self.drones[which] = Drone::Diving(Dive { tick: self.age, aim, swing });
            }
            self.dives += 1;
            self.call = CALL;
        }

        let struck = (0..DRONES).any(|which| {
            self.position(which).is_some_and(|(x, y)| {
                (x - self.ship).abs() <= DRONE_HALF + sky::SHIP_HALF * 0.6
                    && y + DRONE_HALF >= SHIP_Y - sky::SHIP_TALL
                    && y - DRONE_HALF <= SHIP_Y
            })
        });
        if struck { Outcome::Lost } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        sky::night(frame);
        for which in 0..DRONES {
            let Some((x, y)) = self.position(which) else { continue };
            let tint = if which < ACROSS { BACK } else { FRONT };
            frame.poly(
                &[[x - DRONE_HALF, y - 6.0], [x + DRONE_HALF, y - 6.0], [x, y + DRONE_HALF]],
                tint,
            );
            frame.disc(x, y - 4.0, 7.0, tint);
            frame.rect(x - 4.0, y - 7.0, 3.0, 3.0, palette::BACKDROP);
            frame.rect(x + 1.0, y - 7.0, 3.0, 3.0, palette::BACKDROP);
        }
        sky::ship(frame, self.ship, SHIP_Y);
        for (x, y) in &self.shots {
            sky::shot(frame, *x, *y);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/galaxy.rs"]
mod tests;
