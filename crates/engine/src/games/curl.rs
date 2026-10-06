//! Three stones down the ice: knock the red one out of the house and leave all three of yours in it.

use crate::aim;
use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "curl",
    title: "Curling Cast",
    brief: "A red stone sits on the middle of the house. You have three stones to throw. Hold \
            the left button on the stone at the near end, drag back from it and let go, and \
            it slides away and slows as it goes; the dotted line says where it would stop if \
            nothing were done. While a stone is sliding, clicking the left button over and \
            over sweeps ahead of it, which cuts the friction a little and carries it farther. \
            Once all three have come to rest the red stone must be clear of the house and all \
            three of yours must be touching it. Anything else and the round is over.",
    controls: &[
        "hold the left button on the near stone and drag back, then let go",
        "click over and over while a stone slides to sweep",
    ],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Aim, Tag::Physics, Tag::Overhead, Tag::Drag],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Curl::new())
}

pub(crate) const STONES: usize = 3;
pub(crate) const STONE: f32 = 14.0;
pub(crate) const HOUSE: (f32, f32) = (430.0, 240.0);
pub(crate) const HOUSE_R: f32 = 92.0;
pub(crate) const HACK: (f32, f32) = (130.0, 240.0);
/// How fast a stone loses speed on bare ice, a flat rate in pixels a second a
/// second rather than a share of its speed, and the share of that rate left
/// when the ice ahead of it is being swept.
pub(crate) const RUB: f32 = 44.0;
const SWEPT: f32 = 0.7;
pub(crate) const PULL: f32 = 2.8;
const FASTEST: f32 = 360.0;
const KNOCK: f32 = 0.92;
const RAIL: f32 = 0.55;
/// The red stone's body and grip, darker than the red ring of the house it sits on.
const GRANITE: Rgb = Rgb(118, 26, 34);
const GRIP: Rgb = Rgb(232, 206, 196);
const GRAB: f32 = 40.0;
/// How quickly a sweep runs out, so that keeping it up means going on clicking.
const FADE: f32 = 3.0;
const REST: f32 = 3.0;

const LEFT: f32 = 8.0;
const RIGHT: f32 = frame::WIDTH as f32 - 8.0;
const TOP: f32 = 8.0;
const BOTTOM: f32 = frame::HEIGHT as f32 - 8.0;
/// Past this line a stone has run off the end of the sheet and is out of play.
const BACK: f32 = RIGHT - 28.0;

#[derive(Clone, Copy)]
pub(crate) struct Stone {
    pub at: (f32, f32),
    pub pace: (f32, f32),
    pub mine: bool,
}

pub(crate) fn on_house(at: (f32, f32)) -> bool {
    libm::hypotf(at.0 - HOUSE.0, at.1 - HOUSE.1) <= HOUSE_R + STONE
}

pub(crate) struct Curl {
    pub(crate) stones: Vec<Stone>,
    thrown: usize,
    drawing: Option<(f32, f32)>,
    sweep: f32,
}

/// How far a stone thrown this fast would run before it stopped, with nobody
/// sweeping ahead of it.
fn reach(pace: (f32, f32)) -> f32 {
    let fast = libm::hypotf(pace.0, pace.1);
    fast * fast / (2.0 * RUB)
}

impl Curl {
    pub(crate) fn new() -> Self {
        Curl {
            stones: vec![Stone { at: HOUSE, pace: (0.0, 0.0), mine: false }],
            thrown: 0,
            drawing: None,
            sweep: 0.0,
        }
    }

    fn sliding(&self) -> bool {
        self.stones.iter().any(|stone| libm::hypotf(stone.pace.0, stone.pace.1) > REST)
    }

    /// The speed a stone would leave with, were the sling let go of here.
    fn cast_from(&self, x: f32, y: f32) -> (f32, f32) {
        aim::cast(HACK, (x, y), PULL, FASTEST)
    }

    fn slide(&mut self) {
        let rub = if self.sweep > 0.0 { RUB * SWEPT } else { RUB };
        for stone in &mut self.stones {
            let fast = libm::hypotf(stone.pace.0, stone.pace.1);
            if fast <= REST {
                stone.pace = (0.0, 0.0);
                continue;
            }
            let left = (fast - rub * TICK).max(0.0) / fast;
            stone.pace = (stone.pace.0 * left, stone.pace.1 * left);
            stone.at.0 += stone.pace.0 * TICK;
            stone.at.1 += stone.pace.1 * TICK;
            for (spot, pace, low, high) in [
                (&mut stone.at.0, &mut stone.pace.0, LEFT, BACK + STONE * 2.0),
                (&mut stone.at.1, &mut stone.pace.1, TOP, BOTTOM),
            ] {
                walk::bounced(spot, pace, low, high, STONE, RAIL);
            }
        }

        self.stones.retain(|stone| stone.at.0 < BACK);

        for one in 0..self.stones.len() {
            for two in one + 1..self.stones.len() {
                let (mut a, mut b) = (self.stones[one], self.stones[two]);
                walk::bumped((&mut a.at, &mut a.pace), (&mut b.at, &mut b.pace), STONE, KNOCK);
                self.stones[one] = a;
                self.stones[two] = b;
            }
        }
    }
}

impl Game for Curl {
    fn update(&mut self, input: &Input) -> Outcome {
        self.sweep = (self.sweep - FADE * TICK).max(0.0);
        if self.sliding() {
            if input.button_pressed(Button::Left) {
                self.sweep = (self.sweep + 1.0).min(2.0);
            }
            self.slide();
            return Outcome::Playing;
        }
        self.slide();

        if self.thrown == STONES {
            let enemy = self.stones.iter().any(|stone| !stone.mine && on_house(stone.at));
            let mine = self.stones.iter().filter(|stone| stone.mine && on_house(stone.at)).count();
            return if !enemy && mine == STONES { Outcome::Won } else { Outcome::Lost };
        }

        if let Some((x, y)) = aim::drawn(input, HACK, GRAB, &mut self.drawing) {
            let pace = self.cast_from(x, y);
            if libm::hypotf(pace.0, pace.1) > 40.0 {
                self.stones.push(Stone { at: HACK, pace, mine: true });
                self.thrown += 1;
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let ice = Rgb(28, 40, 56);
        frame.rect(LEFT, TOP, RIGHT - LEFT, BOTTOM - TOP, ice);
        frame.rect(BACK, TOP, RIGHT - BACK, BOTTOM - TOP, palette::PANEL);
        frame.line(BACK, TOP, BACK, BOTTOM, 2.0, palette::EDGE);
        frame.line(LEFT, HOUSE.1, RIGHT, HOUSE.1, 1.0, palette::EDGE);
        for (wide, tint) in [
            (HOUSE_R, palette::PLAYER.mix(ice, 0.62)),
            (HOUSE_R * 0.68, ice),
            (HOUSE_R * 0.36, palette::BAD.mix(ice, 0.6)),
            (10.0, ice),
        ] {
            frame.disc(HOUSE.0, HOUSE.1, wide, tint);
        }
        frame.ring(HOUSE.0, HOUSE.1, HOUSE_R, 2.0, palette::MUTED);

        if let Some((x, y)) = self.drawing {
            let pace = self.cast_from(x, y);
            let fast = libm::hypotf(pace.0, pace.1);
            frame.line(HACK.0, HACK.1, x, y, 3.0, palette::MUTED);
            if fast > 0.0 {
                let far = reach(pace);
                let mut along = 24.0;
                while along < far {
                    frame.disc(
                        HACK.0 + pace.0 / fast * along,
                        HACK.1 + pace.1 / fast * along,
                        2.5,
                        palette::EDGE,
                    );
                    along += 24.0;
                }
            }
        }

        for stone in &self.stones {
            let (body, grip) =
                if stone.mine { (palette::PLAYER, palette::MARK) } else { (GRANITE, GRIP) };
            frame.disc(stone.at.0, stone.at.1, STONE, body);
            frame.ring(stone.at.0, stone.at.1, STONE * 0.5, 3.0, grip);
        }
        if self.thrown < STONES && !self.sliding() {
            frame.disc(HACK.0, HACK.1, STONE, palette::PLAYER);
            frame.ring(HACK.0, HACK.1, STONE + 6.0, 2.0, palette::MARK);
        }
        if self.sweep > 0.0 {
            frame.text_center(frame::WIDTH as f32 * 0.5, 14.0, "SWEEPING", 1, palette::WARN);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/curl.rs"]
mod tests;
