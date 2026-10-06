//! Keep a ball out of the wall behind you while it gets faster.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "volley",
    title: "Very Volley",
    brief: "A ball crosses the court, and every rally it comes back faster. Move the bat on \
            the left to keep it out of the wall behind you for thirty seconds. Let it past \
            once and the round is over. The bat on the right never misses, so the ball \
            always comes back.",
    controls: &["UP and DOWN move the bat"],
    pointer: false,
    scoring: Scoring::Boolean,
    varies: true,
    tags: &[Tag::Fixed, Tag::Lethal, Tag::Timing, Tag::Physics],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Volley::new(seed))
}

const SURVIVE: u32 = 30 * TICK_HZ;
const TOP: f32 = 38.0;
const BOTTOM: f32 = 446.0;
const WALL: f32 = 22.0;
const FAR: f32 = 618.0;
const BAT_WIDE: f32 = 10.0;
const BAT_TALL: f32 = 76.0;
const MINE: f32 = 34.0;
const THEIRS: f32 = 596.0;
const BALL: f32 = 8.0;
const MY_PACE: f32 = 250.0;
const OPENING: f32 = 330.0;
const QUICKENING: f32 = 9.5;
const SERVE_REST: u32 = 36;

struct Volley {
    rng: Rng,
    age: u32,
    rest: u32,
    ball: (f32, f32),
    drift: (f32, f32),
    mine: f32,
    theirs: f32,
}

impl Volley {
    fn new(seed: u64) -> Self {
        let middle = (TOP + BOTTOM) * 0.5;
        let mut volley = Volley {
            rng: Rng::new(seed),
            age: 0,
            rest: SERVE_REST,
            ball: (frame::WIDTH as f32 * 0.5, middle),
            drift: (0.0, 0.0),
            mine: middle,
            theirs: middle,
        };
        volley.serve();
        volley
    }

    fn pace(&self) -> f32 {
        OPENING + QUICKENING * (self.age as f32 * TICK)
    }

    fn serve(&mut self) {
        let middle = (TOP + BOTTOM) * 0.5;
        self.ball = (frame::WIDTH as f32 * 0.5, middle);
        let slope = self.rng.range(-0.5, 0.5);
        let length = (1.0 + slope * slope).sqrt();
        self.drift = (-self.pace() / length, self.pace() * slope / length);
        self.rest = SERVE_REST;
    }

    /// Playing, or won once the round has been stood out.
    fn standing(&self) -> Outcome {
        if self.age >= SURVIVE { Outcome::Won } else { Outcome::Playing }
    }

    /// Where the bat sends it back, steeper the farther from the middle it lands.
    fn returned(&mut self, bat: f32, away: f32) {
        let offset = ((self.ball.1 - bat) / (BAT_TALL * 0.5)).clamp(-1.0, 1.0);
        let slope = offset * 0.9;
        let length = (1.0 + slope * slope).sqrt();
        self.drift = (away * self.pace() / length, self.pace() * slope / length);
    }
}

impl Game for Volley {
    fn update(&mut self, input: &Input) -> Outcome {
        self.age += 1;
        let lean = input.axis(Key::Up, Key::Down);
        let half = BAT_TALL * 0.5;
        self.mine = (self.mine + lean * MY_PACE * TICK).clamp(TOP + half, BOTTOM - half);
        self.theirs = self.ball.1.clamp(TOP + half, BOTTOM - half);

        if self.rest > 0 {
            self.rest -= 1;
            return self.standing();
        }

        let was = self.ball.0;
        self.ball.0 += self.drift.0 * TICK;
        self.ball.1 += self.drift.1 * TICK;
        walk::bounced(&mut self.ball.1, &mut self.drift.1, TOP, BOTTOM, BALL, 1.0);

        // The tick the ball crosses the face of a bat, not every tick it is
        // past it, so a bat slid across afterward cannot fetch it back.
        let my_face = MINE + BAT_WIDE + BALL;
        if self.drift.0 < 0.0
            && was > my_face
            && self.ball.0 <= my_face
            && (self.ball.1 - self.mine).abs() <= half + BALL
        {
            self.ball.0 = my_face;
            self.returned(self.mine, 1.0);
        }
        let their_face = THEIRS - BALL;
        if self.drift.0 > 0.0
            && was < their_face
            && self.ball.0 >= their_face
            && (self.ball.1 - self.theirs).abs() <= half + BALL
        {
            self.ball.0 = their_face;
            self.returned(self.theirs, -1.0);
        }

        if self.ball.0 - BALL < WALL {
            return Outcome::Lost;
        }
        if self.ball.0 + BALL > FAR {
            self.serve();
        }
        self.standing()
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        frame.rect(WALL - 6.0, TOP - 4.0, FAR - WALL + 12.0, 4.0, palette::EDGE);
        frame.rect(WALL - 6.0, BOTTOM, FAR - WALL + 12.0, 4.0, palette::EDGE);
        frame.rect(WALL - 6.0, TOP, 6.0, BOTTOM - TOP, palette::BAD);
        frame.rect(FAR, TOP, 6.0, BOTTOM - TOP, palette::EDGE);
        let mut mark = TOP;
        while mark < BOTTOM {
            frame.rect(width * 0.5 - 1.0, mark, 2.0, 12.0, palette::PANEL);
            mark += 24.0;
        }

        let half = BAT_TALL * 0.5;
        frame.rect(MINE, self.mine - half, BAT_WIDE, BAT_TALL, palette::PLAYER);
        frame.rect(THEIRS, self.theirs - half, BAT_WIDE, BAT_TALL, palette::MUTED);
        frame.disc(self.ball.0, self.ball.1, BALL, palette::INK);
    }
}

#[cfg(test)]
#[path = "../tests/games/volley.rs"]
mod tests;
