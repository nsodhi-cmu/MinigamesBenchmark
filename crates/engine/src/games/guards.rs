//! Cross dark rooms past patrolling guards, unseen while standing still.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::palette;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "guards",
    title: "Guard Gauntlet",
    brief: "Cross three dark rooms from the left to the lit doorway on the right. Guards walk \
            the same rounds over and over, and each one lights a cone in front of it. While you \
            move you can be seen. Once you stop you fade, and after a moment you are gone from \
            sight altogether, even standing in a guard's light. A guard that sees you before \
            you have faded ends the round. Walls block a guard's light.",
    controls: &["arrow keys move", "let go of every arrow to fade from sight"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Hostile, Tag::Navigate, Tag::Timing, Tag::Overhead],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Guards::new())
}

const TOP: f32 = 0.0;
const RIGHT: f32 = frame::WIDTH as f32;
const BOTTOM: f32 = frame::HEIGHT as f32;
pub(crate) const PACE: f32 = 80.0;
pub(crate) const BODY: f32 = 6.0;
/// Ticks of standing still before the player is out of sight.
pub(crate) const FADE: u32 = 30;
const REACH: f32 = 140.0;
const HALF_ANGLE: f32 = 30.0;
const GUARD: f32 = 8.0;

const LIGHT: Rgb = Rgb(58, 54, 34);
const CAUGHT: Rgb = Rgb(110, 34, 34);
const COAT: Rgb = Rgb(150, 150, 164);
const SNEAK: Rgb = Rgb(96, 150, 220);
const DOOR: Rgb = Rgb(70, 120, 80);

/// A walk around a closed loop of points, turning on the spot at each one for
/// `rest` ticks.
pub(crate) struct Patrol {
    pub(crate) points: &'static [(f32, f32)],
    pub(crate) rest: u32,
}

pub(crate) struct Room {
    /// Every guard in a room comes back to where it started after this many ticks.
    pub(crate) period: u32,
    /// Left, top, width, height.
    pub(crate) walls: &'static [(f32, f32, f32, f32)],
    pub(crate) patrols: &'static [Patrol],
    pub(crate) start: (f32, f32),
    pub(crate) door: (f32, f32, f32, f32),
}

pub(crate) const ROOMS: [Room; 3] = [
    Room {
        period: 360,
        walls: &[
            (250.0, 127.0, 44.0, 44.0),
            (250.0, 317.0, 44.0, 44.0),
            (380.0, 223.0, 44.0, 44.0),
        ],
        patrols: &[
            Patrol { points: &[(200.0, 47.0), (200.0, 437.0)], rest: 24 },
            Patrol { points: &[(460.0, 437.0), (460.0, 47.0)], rest: 24 },
        ],
        start: (24.0, 240.0),
        door: (620.0, 187.0, 20.0, 106.0),
    },
    Room {
        period: 360,
        walls: &[(100.0, 0.0, 440.0, 137.0), (100.0, 343.0, 440.0, 137.0)],
        patrols: &[
            Patrol {
                points: &[(250.0, 240.0), (251.0, 240.0), (250.0, 240.0), (251.0, 240.0)],
                rest: 88,
            },
            Patrol {
                points: &[(390.0, 240.0), (391.0, 240.0), (390.0, 240.0), (391.0, 240.0)],
                rest: 88,
            },
            Patrol { points: &[(580.0, 137.0), (580.0, 343.0)], rest: 40 },
        ],
        start: (24.0, 240.0),
        door: (620.0, 190.0, 20.0, 100.0),
    },
    Room {
        period: 480,
        walls: &[
            (0.0, 157.0, 260.0, 16.0),
            (320.0, 157.0, 320.0, 16.0),
            (0.0, 317.0, 380.0, 16.0),
            (440.0, 317.0, 200.0, 16.0),
        ],
        patrols: &[
            Patrol { points: &[(40.0, 406.0), (600.0, 406.0)], rest: 40 },
            Patrol {
                points: &[(410.0, 245.0), (411.0, 245.0), (410.0, 245.0), (411.0, 245.0)],
                rest: 119,
            },
            Patrol { points: &[(600.0, 78.0), (60.0, 78.0)], rest: 40 },
        ],
        start: (24.0, 406.0),
        door: (620.0, 33.0, 20.0, 90.0),
    },
];

fn heading(from: (f32, f32), to: (f32, f32)) -> f32 {
    libm::atan2f(to.1 - from.1, to.0 - from.0).to_degrees()
}

fn turned(from: f32, to: f32) -> f32 {
    let change = (to - from).rem_euclid(360.0);
    if change > 180.0 { change - 360.0 } else { change }
}

/// Where a guard is and which way it faces, in degrees, `tick` ticks into its round.
pub(crate) fn guard_at(patrol: &Patrol, period: u32, tick: u32) -> ((f32, f32), f32) {
    let points = patrol.points;
    let legs = points.len();
    let length = |leg: usize| {
        let (a, b) = (points[leg], points[(leg + 1) % legs]);
        ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
    };
    let walking = (period - patrol.rest * legs as u32) as f32;
    let total: f32 = (0..legs).map(length).sum();
    let mut left = (tick % period) as f32;
    for leg in 0..legs {
        let (a, b) = (points[leg], points[(leg + 1) % legs]);
        let face = heading(a, b);
        let along = length(leg) / total * walking;
        if left < along {
            let t = left / along;
            return ((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t), face);
        }
        left -= along;
        if left < patrol.rest as f32 {
            let next = heading(b, points[(leg + 2) % legs]);
            return (b, face + turned(face, next) * left / patrol.rest as f32);
        }
        left -= patrol.rest as f32;
    }
    (points[0], heading(points[0], points[1 % legs]))
}

/// Where a line from `a` to `b` enters and leaves a box, each as a fraction of
/// the way along it. Nothing where the line misses the box.
fn through(a: (f32, f32), b: (f32, f32), box_: (f32, f32, f32, f32)) -> Option<(f32, f32)> {
    let (x, y, w, h) = box_;
    let (mut low, mut high) = (0.0f32, 1.0f32);
    for (from, delta, min, max) in [(a.0, b.0 - a.0, x, x + w), (a.1, b.1 - a.1, y, y + h)] {
        if delta.abs() < 1e-6 {
            if from < min || from > max {
                return None;
            }
        } else {
            let (t0, t1) = ((min - from) / delta, (max - from) / delta);
            low = low.max(t0.min(t1));
            high = high.min(t0.max(t1));
        }
    }
    (low <= high).then_some((low, high))
}

/// Whether the straight line between two points passes through a wall.
pub(crate) fn blocked(room: &Room, a: (f32, f32), b: (f32, f32)) -> bool {
    room.walls.iter().any(|wall| through(a, b, *wall).is_some())
}

/// How far along a line from `a` to `b` the light gets before a wall stops it
/// or it leaves the room, as a fraction of the way.
fn reaches(room: &Room, a: (f32, f32), b: (f32, f32)) -> f32 {
    let leaves = through(a, b, (0.0, TOP, RIGHT, BOTTOM - TOP)).map_or(0.0, |(_, out)| out);
    room.walls
        .iter()
        .filter_map(|wall| through(a, b, *wall).map(|(into, _)| into))
        .fold(leaves, f32::min)
}

/// Whether a guard at `at` facing `face` sees the point `spot`.
pub(crate) fn sees(room: &Room, at: (f32, f32), face: f32, spot: (f32, f32)) -> bool {
    let (dx, dy) = (spot.0 - at.0, spot.1 - at.1);
    (dx * dx + dy * dy).sqrt() <= REACH
        && turned(face, libm::atan2f(dy, dx).to_degrees()).abs() <= HALF_ANGLE
        && !blocked(room, at, spot)
}

/// Whether a body standing at a point is clear of the walls and inside the room.
pub(crate) fn open(room: &Room, x: f32, y: f32) -> bool {
    (BODY..=RIGHT - BODY).contains(&x)
        && (TOP + BODY..=BOTTOM - BODY).contains(&y)
        && room.walls.iter().all(|&(wx, wy, w, h)| {
            x < wx - BODY || x > wx + w + BODY || y < wy - BODY || y > wy + h + BODY
        })
}

#[derive(Clone)]
pub(crate) struct Guards {
    pub(crate) room: usize,
    pub(crate) tick: u32,
    pub(crate) at: (f32, f32),
    /// Ticks since the player last moved.
    pub(crate) still: u32,
    caught: Option<usize>,
}

impl Guards {
    pub(crate) fn new() -> Self {
        Guards::entering(0)
    }

    pub(crate) fn entering(room: usize) -> Self {
        Guards { room, tick: 0, at: ROOMS[room].start, still: 0, caught: None }
    }
}

impl Game for Guards {
    fn update(&mut self, input: &Input) -> Outcome {
        let room = &ROOMS[self.room];
        self.tick += 1;
        let heading = input.heading();
        if let Some(step) = heading {
            self.at = walk::walked(self.at, step, PACE * TICK, |x, y| open(room, x, y));
        }
        self.still = if heading.is_some() { 0 } else { self.still + 1 };

        let (x, y, w, h) = room.door;
        if (x..x + w).contains(&self.at.0) && (y..y + h).contains(&self.at.1) {
            if self.room + 1 == ROOMS.len() {
                return Outcome::Won;
            }
            *self = Guards::entering(self.room + 1);
            return Outcome::Playing;
        }
        if self.still < FADE {
            self.caught = room.patrols.iter().position(|patrol| {
                let (guard, face) = guard_at(patrol, room.period, self.tick);
                sees(room, guard, face, self.at)
            });
        }
        if self.caught.is_some() { Outcome::Lost } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let room = &ROOMS[self.room];
        frame.rect(0.0, TOP, RIGHT, BOTTOM - TOP, palette::BACKDROP);
        let (x, y, w, h) = room.door;
        frame.rect(x, y, w, h, DOOR);

        for (which, patrol) in room.patrols.iter().enumerate() {
            let (guard, face) = guard_at(patrol, room.period, self.tick);
            let mut fan = vec![[guard.0, guard.1]];
            for ray in 0..=24 {
                let angle = (face - HALF_ANGLE + HALF_ANGLE * 2.0 * ray as f32 / 24.0).to_radians();
                let far =
                    (guard.0 + libm::cosf(angle) * REACH, guard.1 + libm::sinf(angle) * REACH);
                let reach = reaches(room, guard, far);
                fan.push([
                    guard.0 + (far.0 - guard.0) * reach,
                    guard.1 + (far.1 - guard.1) * reach,
                ]);
            }
            let tint = if self.caught == Some(which) { CAUGHT } else { LIGHT };
            frame.poly(&fan, tint);
        }
        for &(wx, wy, ww, wh) in room.walls {
            frame.rect(wx, wy, ww, wh, palette::PANEL);
        }
        for patrol in room.patrols {
            let (guard, face) = guard_at(patrol, room.period, self.tick);
            let a = face.to_radians();
            frame.disc(guard.0, guard.1, GUARD, COAT);
            frame.disc(guard.0 + libm::cosf(a) * GUARD, guard.1 + libm::sinf(a) * GUARD, 3.0, COAT);
        }

        let shown = 1.0 - self.still.min(FADE) as f32 / FADE as f32;
        let tint = palette::BACKDROP.mix(SNEAK, 0.08 + 0.92 * shown);
        frame.disc(self.at.0, self.at.1, BODY, tint);
    }
}

#[cfg(test)]
#[path = "../tests/games/guards.rs"]
mod tests;
