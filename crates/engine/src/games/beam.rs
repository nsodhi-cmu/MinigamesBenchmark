//! Turn the one mirror until the beam finds its way from portal to portal.

use crate::frame::Frame;
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "beam",
    title: "Beam Bounce",
    brief: "A beam leaves the near portal along a line that never changes and lands on the \
            mirror in the middle of the room. LEFT and RIGHT turn that mirror. \
            Press SPACE and the beam runs: it turns off the mirror and travels on until \
            something stops it. Get it into the far portal. A beam that runs into a wall \
            costs nothing but the time. A beam that runs into one of the posts standing in \
            the room ends the round. Five rooms finish it.",
    controls: &[
        "LEFT and RIGHT turn the mirror",
        "SPACE sends the beam, and calls back one still running",
    ],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Puzzle],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Beam::new())
}

pub(crate) const ROOMS: usize = 5;
/// Half the length of the mirror.
const ARM: f32 = 30.0;
pub(crate) const PORTAL: f32 = 26.0;
/// How fast the mirror turns: slowly for the first touch of an arrow, working
/// up to a sweep if it is held.
const TURN_FIRST: f32 = 0.2;
const TURN_SWEEP: f32 = 2.4;
const TURN_WORKS_UP_IN: f32 = 0.55;
/// How fast the beam travels once it is sent, which is fast enough that it is
/// there almost at once.
const CREEP: f32 = 2600.0;
const LEGS: usize = 3;

pub(crate) struct Room {
    pub wall: &'static [[f32; 2]],
    pub posts: &'static [&'static [[f32; 2]]],
    pub from: [f32; 2],
    pub mirror: [f32; 2],
    pub into: [f32; 2],
    pub turn: f32,
}

pub(crate) const ROOMS_LAID: [Room; ROOMS] = [
    Room {
        wall: &[[64.0, 58.0], [566.0, 44.0], [592.0, 392.0], [88.0, 438.0]],
        posts: &[],
        from: [148.0, 132.0],
        mirror: [304.0, 258.0],
        into: [460.0, 140.0],
        turn: 0.4,
    },
    Room {
        wall: &[[52.0, 72.0], [430.0, 46.0], [604.0, 214.0], [512.0, 428.0], [104.0, 400.0]],
        posts: &[],
        from: [136.0, 348.0],
        mirror: [318.0, 216.0],
        into: [512.0, 322.0],
        turn: 2.1,
    },
    Room {
        wall: &[[70.0, 50.0], [590.0, 76.0], [548.0, 424.0], [116.0, 396.0]],
        posts: &[&[[300.0, 60.0], [346.0, 66.0], [334.0, 186.0], [288.0, 180.0]]],
        from: [132.0, 118.0],
        mirror: [258.0, 296.0],
        into: [484.0, 138.0],
        turn: 1.0,
    },
    Room {
        wall: &[[58.0, 96.0], [258.0, 44.0], [598.0, 130.0], [560.0, 408.0], [96.0, 436.0]],
        posts: &[
            &[[253.0, 95.0], [315.0, 103.0], [305.0, 163.0], [243.0, 155.0]],
            &[[305.0, 334.0], [393.0, 349.0], [385.0, 400.0], [297.0, 385.0]],
        ],
        from: [130.0, 140.0],
        mirror: [170.0, 330.0],
        into: [500.0, 225.0],
        turn: 1.9,
    },
    Room {
        wall: &[
            [96.0, 52.0],
            [512.0, 62.0],
            [604.0, 246.0],
            [470.0, 434.0],
            [148.0, 420.0],
            [56.0, 232.0],
        ],
        posts: &[
            &[[244.0, 106.0], [289.0, 111.0], [284.0, 150.0], [239.0, 145.0]],
            &[[300.0, 300.0], [390.0, 316.0], [380.0, 376.0], [290.0, 360.0]],
            &[[420.0, 120.0], [480.0, 130.0], [470.0, 196.0], [410.0, 186.0]],
        ],
        from: [150.0, 380.0],
        mirror: [130.0, 120.0],
        into: [462.0, 309.0],
        turn: 0.6,
    },
];

type Spot = (f32, f32);

fn spot(p: [f32; 2]) -> Spot {
    (p[0], p[1])
}

/// Where a ray meets a length of wall, as a distance along the ray.
fn meets_line(at: Spot, way: Spot, a: Spot, b: Spot) -> Option<f32> {
    let edge = (b.0 - a.0, b.1 - a.1);
    let denom = way.0 * edge.1 - way.1 * edge.0;
    if denom.abs() < 1e-6 {
        return None;
    }
    let gap = (a.0 - at.0, a.1 - at.1);
    let along = (gap.0 * edge.1 - gap.1 * edge.0) / denom;
    let across = (gap.0 * way.1 - gap.1 * way.0) / denom;
    (along > 0.01 && (0.0..=1.0).contains(&across)).then_some(along)
}

/// Where a ray first meets a portal, as a distance along the ray.
fn meets_ring(at: Spot, way: Spot, middle: Spot, wide: f32) -> Option<f32> {
    let gap = (at.0 - middle.0, at.1 - middle.1);
    let half = gap.0 * way.0 + gap.1 * way.1;
    let over = gap.0 * gap.0 + gap.1 * gap.1 - wide * wide;
    let under = half * half - over;
    if under < 0.0 {
        return None;
    }
    let near = -half - under.sqrt();
    (near > 0.01).then_some(near)
}

fn edges_of(ring: &[[f32; 2]]) -> impl Iterator<Item = (Spot, Spot)> + '_ {
    (0..ring.len()).map(move |i| (spot(ring[i]), spot(ring[(i + 1) % ring.len()])))
}

/// What the beam came to rest against.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Ends {
    Wall,
    Post,
    Portal,
}

pub(crate) struct Shot {
    pub bends: Vec<Spot>,
    pub ends: Ends,
}

impl Shot {
    pub fn far(&self) -> f32 {
        self.bends
            .windows(2)
            .map(|leg| libm::hypotf(leg[1].0 - leg[0].0, leg[1].1 - leg[0].1))
            .sum()
    }

    /// Where the head of the beam is once it has run this far.
    fn head(&self, run: f32) -> Spot {
        let mut left = run;
        for leg in self.bends.windows(2) {
            let long = libm::hypotf(leg[1].0 - leg[0].0, leg[1].1 - leg[0].1);
            if left <= long {
                let part = if long == 0.0 { 0.0 } else { left / long };
                return (
                    leg[0].0 + (leg[1].0 - leg[0].0) * part,
                    leg[0].1 + (leg[1].1 - leg[0].1) * part,
                );
            }
            left -= long;
        }
        *self.bends.last().expect("a beam starts somewhere")
    }
}

/// The whole run of the beam with the mirror laid at this angle.
pub(crate) fn trace(room: &Room, turn: f32) -> Shot {
    let (sin, cos) = libm::sincosf(turn);
    let middle = spot(room.mirror);
    let mirror = (
        (middle.0 - cos * ARM, middle.1 - sin * ARM),
        (middle.0 + cos * ARM, middle.1 + sin * ARM),
    );
    let mut at = spot(room.from);
    let mut way = (middle.0 - at.0, middle.1 - at.1);
    let long = libm::hypotf(way.0, way.1);
    way = (way.0 / long, way.1 / long);

    let mut bends = vec![at];
    for leg in 0..LEGS {
        let mut nearest = f32::MAX;
        let mut on_mirror = false;
        let mut ends = Ends::Wall;
        for (a, b) in edges_of(room.wall) {
            if let Some(far) = meets_line(at, way, a, b)
                && far < nearest
            {
                (nearest, on_mirror, ends) = (far, false, Ends::Wall);
            }
        }
        for (a, b) in room.posts.iter().flat_map(|post| edges_of(post)) {
            if let Some(far) = meets_line(at, way, a, b)
                && far < nearest
            {
                (nearest, on_mirror, ends) = (far, false, Ends::Post);
            }
        }
        if leg == 0
            && let Some(far) = meets_line(at, way, mirror.0, mirror.1)
            && far < nearest
        {
            (nearest, on_mirror, ends) = (far, true, Ends::Wall);
        }
        if let Some(far) = meets_ring(at, way, spot(room.into), PORTAL)
            && far < nearest
        {
            (nearest, on_mirror, ends) = (far, false, Ends::Portal);
        }
        if nearest == f32::MAX {
            break;
        }
        at = (at.0 + way.0 * nearest, at.1 + way.1 * nearest);
        bends.push(at);
        if !on_mirror {
            return Shot { bends, ends };
        }
        let face = (mirror.1.0 - mirror.0.0, mirror.1.1 - mirror.0.1);
        let side = (-face.1, face.0);
        let unit = libm::hypotf(side.0, side.1);
        let side = (side.0 / unit, side.1 / unit);
        let dot = way.0 * side.0 + way.1 * side.1;
        way = (way.0 - 2.0 * dot * side.0, way.1 - 2.0 * dot * side.1);
        at = (at.0 + way.0 * 0.05, at.1 + way.1 * 0.05);
    }
    Shot { bends, ends: Ends::Wall }
}

struct Beam {
    room: usize,
    turn: f32,
    /// How long an arrow has been held, which is what the mirror speeds up on.
    holding: f32,
    /// The beam in flight: the run it was traced on when it left, and how far
    /// along that run its head has gotten. Traced once, because the mirror it
    /// bounced off is the mirror as it stood when the beam was sent.
    run: Option<(Shot, f32)>,
}

impl Beam {
    fn new() -> Self {
        Beam { room: 0, turn: ROOMS_LAID[0].turn, holding: 0.0, run: None }
    }
}

impl Game for Beam {
    fn update(&mut self, input: &Input) -> Outcome {
        // The mirror is only turned between shots. While a beam is running it
        // is off the mirror already, so turning it would bend light that has
        // gone by.
        if self.run.is_none() {
            let way = input.axis(Key::Left, Key::Right);
            self.holding = if way == 0.0 { 0.0 } else { self.holding + TICK };
            let worked_up = (self.holding / TURN_WORKS_UP_IN).min(1.0);
            self.turn += way * (TURN_FIRST + (TURN_SWEEP - TURN_FIRST) * worked_up) * TICK;
        }
        if input.pressed(Key::Space) {
            self.run = match self.run {
                Some(_) => None,
                None => Some((trace(&ROOMS_LAID[self.room], self.turn), 0.0)),
            };
        }

        let Some((shot, run)) = self.run.take() else {
            return Outcome::Playing;
        };
        let run = run + CREEP * TICK;
        if run < shot.far() {
            self.run = Some((shot, run));
            return Outcome::Playing;
        }
        match shot.ends {
            Ends::Post => Outcome::Lost,
            // A beam that only finds a wall costs the time it took and no more,
            // so the mirror can be tried again from where it was left.
            Ends::Wall => {
                self.run = None;
                Outcome::Playing
            }
            Ends::Portal if self.room + 1 == ROOMS => Outcome::Won,
            Ends::Portal => {
                self.room += 1;
                self.turn = ROOMS_LAID[self.room].turn;
                self.run = None;
                Outcome::Playing
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let room = &ROOMS_LAID[self.room];
        for (a, b) in edges_of(room.wall) {
            frame.line(a.0, a.1, b.0, b.1, 3.0, palette::MUTED);
        }
        for post in room.posts {
            frame.poly(post, palette::BAD.mix(palette::BACKDROP, 0.74));
            for (a, b) in edges_of(post) {
                frame.line(a.0, a.1, b.0, b.1, 3.0, palette::BAD);
            }
        }

        let (from, into) = (spot(room.from), spot(room.into));
        frame.ring(into.0, into.1, PORTAL, 3.0, palette::GOOD);
        frame.ring(into.0, into.1, PORTAL - 7.0, 2.0, palette::GOOD.mix(palette::BACKDROP, 0.5));
        frame.ring(from.0, from.1, PORTAL - 6.0, 3.0, palette::PINK);

        let middle = spot(room.mirror);
        frame.ring(middle.0, middle.1, ARM, 1.0, palette::EDGE);
        let (sin, cos) = libm::sincosf(self.turn);
        frame.line(
            middle.0 - cos * ARM,
            middle.1 - sin * ARM,
            middle.0 + cos * ARM,
            middle.1 + sin * ARM,
            5.0,
            palette::MARK,
        );

        // The first leg never changes, so it is always shown; the rest of the
        // run is the beam as it was sent.
        frame.line(
            from.0,
            from.1,
            middle.0,
            middle.1,
            1.0,
            palette::BAD.mix(palette::BACKDROP, 0.55),
        );
        if let Some((shot, run)) = &self.run {
            let mut left = *run;
            for leg in shot.bends.windows(2) {
                let long = libm::hypotf(leg[1].0 - leg[0].0, leg[1].1 - leg[0].1);
                let part = (left / long.max(0.001)).min(1.0);
                let end = (
                    leg[0].0 + (leg[1].0 - leg[0].0) * part,
                    leg[0].1 + (leg[1].1 - leg[0].1) * part,
                );
                frame.line(leg[0].0, leg[0].1, end.0, end.1, 3.0, palette::BAD);
                if left < long {
                    break;
                }
                left -= long;
            }
            let head = shot.head(*run);
            frame.disc(head.0, head.1, 4.0, palette::WARN);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/beam.rs"]
mod tests;
