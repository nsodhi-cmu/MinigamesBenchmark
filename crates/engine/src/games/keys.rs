//! Eight keys, one of them marked, and half a minute of shuffling to sit through.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK_HZ, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "keys",
    title: "Kept Key",
    brief: "Eight keys hang in a ring and one of them glows for the first three seconds. \
            Once the glow is gone they are all alike. For the next twenty-seven the ring \
            turns and the keys trade places, two pairs at a time or all four pairs at once, \
            one of each pair swinging outside the ring and the other inside. Twenty of these \
            go by. Keep your eye on the one that glowed, and when the shuffling stops, click \
            it. Click any of the other seven and the round is over.",
    controls: &["left click the key that glowed, once the shuffling stops"],
    pointer: true,
    scoring: Scoring::Boolean,
    varies: true,
    tags: &[Tag::Fixed, Tag::Strict, Tag::Memory],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Kept::new(seed))
}

pub(crate) const KEYS: usize = 8;
const MIDDLE: (f32, f32) = (320.0, 252.0);
const RING: f32 = 128.0;
/// How far off the ring a key swings when it trades places with another.
const BOW: f32 = 46.0;
const KEY: f32 = 15.0;
const GRAB: f32 = 26.0;
const BASE: f32 = -std::f32::consts::FRAC_PI_2;
const STEP: f32 = std::f32::consts::TAU / KEYS as f32;
/// Three seconds of glow, then twenty moves over the next twenty-seven, each
/// of them eighty-one ticks long.
const GLOW: u32 = 3 * TICK_HZ;
pub(crate) const MOVES: u32 = 20;
pub(crate) const SHUFFLE: u32 = 27 * TICK_HZ;
const SPAN: u32 = SHUFFLE / MOVES;
const REST: u32 = 15;
const MOVE: u32 = SPAN - REST;

pub(crate) struct Kept {
    rng: Rng,
    spot: [usize; KEYS],
    from: [f32; KEYS],
    to: [f32; KEYS],
    bow: [f32; KEYS],
    pub(crate) marked: usize,
    tick: u32,
    age: u32,
    shuffles: u32,
    pub(crate) picking: bool,
}

impl Kept {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let marked = rng.below(KEYS);
        let resting = std::array::from_fn(|key| BASE + key as f32 * STEP);
        Kept {
            rng,
            spot: std::array::from_fn(|key| key),
            from: resting,
            to: resting,
            bow: [0.0; KEYS],
            marked,
            tick: 0,
            age: MOVE + REST - 1,
            shuffles: 0,
            picking: false,
        }
    }

    /// Where a key hangs at this point in the move it is making.
    pub(crate) fn place(&self, key: usize) -> (f32, f32) {
        let part = (self.age.min(MOVE) as f32 / MOVE as f32).clamp(0.0, 1.0);
        let swing = part * std::f32::consts::PI;
        let eased = 0.5 - 0.5 * libm::cosf(swing);
        let angle = self.from[key] + (self.to[key] - self.from[key]) * eased;
        let out = RING + self.bow[key] * libm::sinf(swing);
        (MIDDLE.0 + libm::cosf(angle) * out, MIDDLE.1 + libm::sinf(angle) * out)
    }

    fn key_in(&self, slot: usize) -> usize {
        (0..KEYS).find(|key| self.spot[*key] == slot).expect("every slot holds a key")
    }

    /// Have one pair of slots trade the keys standing in them, the one going
    /// forward swinging outside the ring and the one coming back inside it.
    fn trade(&mut self, slot: usize, apart: usize) {
        let (here, there) = (self.key_in(slot), self.key_in((slot + apart) % KEYS));
        let swing = apart as f32 * STEP;
        self.to[here] = self.from[here] + swing;
        self.to[there] = self.from[there] - swing;
        self.bow[here] = BOW;
        self.bow[there] = -BOW;
        self.spot.swap(here, there);
    }

    /// Turn the whole ring, trade two pairs at once, or trade all four pairs
    /// of neighbors at once.
    fn deal(&mut self) {
        self.bow = [0.0; KEYS];
        match self.rng.below(3) {
            0 => {
                let steps = [1i32, -1, 2, -2, 3, -3][self.rng.below(6)];
                for key in 0..KEYS {
                    self.spot[key] =
                        (self.spot[key] as i32 + steps).rem_euclid(KEYS as i32) as usize;
                    self.to[key] = self.from[key] + steps as f32 * STEP;
                }
            }
            1 => {
                // Two pairs, kept apart by trading across one half of the ring
                // and then across the other.
                let half = self.rng.below(KEYS / 2);
                let apart = 1 + self.rng.below(2);
                self.trade(half, apart);
                self.trade(half + KEYS / 2, apart);
            }
            _ => {
                let first = self.rng.below(2);
                for pair in 0..KEYS / 2 {
                    self.trade(first + pair * 2, 1);
                }
            }
        }
        self.shuffles += 1;
    }

    fn under(&self, (x, y): (f32, f32)) -> Option<usize> {
        (0..KEYS)
            .map(|key| (key, self.place(key)))
            .map(|(key, at)| (key, libm::hypotf(at.0 - x, at.1 - y)))
            .filter(|(_, off)| *off <= GRAB)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(key, _)| key)
    }
}

/// One key, drawn the same as every other one.
fn key(frame: &mut Frame, at: (f32, f32), tint: Rgb) {
    let (x, y) = at;
    frame.ring(x, y - 8.0, 7.0, 3.0, tint);
    frame.rect(x - 2.0, y - 2.0, 4.0, 16.0, tint);
    frame.rect(x + 2.0, y + 4.0, 5.0, 3.0, tint);
    frame.rect(x + 2.0, y + 10.0, 5.0, 3.0, tint);
}

impl Game for Kept {
    fn update(&mut self, input: &Input) -> Outcome {
        if self.picking {
            if input.button_pressed(Button::Left)
                && let Some(key) = self.under(input.cursor())
            {
                return if key == self.marked { Outcome::Won } else { Outcome::Lost };
            }
            return Outcome::Playing;
        }

        self.tick += 1;
        if self.tick <= GLOW {
            return Outcome::Playing;
        }
        if self.age + 1 < MOVE + REST {
            self.age += 1;
            return Outcome::Playing;
        }
        self.from = self.to;
        self.bow = [0.0; KEYS];
        if self.tick >= GLOW + SHUFFLE {
            self.picking = true;
            return Outcome::Playing;
        }
        self.deal();
        self.age = 0;
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        frame.ring(MIDDLE.0, MIDDLE.1, RING, 1.0, palette::PANEL);
        let showing = self.tick <= GLOW;
        for which in 0..KEYS {
            let at = self.place(which);
            let lit = showing && which == self.marked;
            if lit {
                frame.ring(at.0, at.1 + 2.0, KEY + 8.0, 2.0, palette::WARN);
            }
            key(frame, at, if lit { palette::WARN } else { palette::MUTED });
        }
        if self.picking {
            frame.text_center(
                frame::WIDTH as f32 * 0.5,
                10.0,
                "CLICK THE KEY THAT GLOWED",
                1,
                palette::MARK,
            );
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/keys.rs"]
mod tests;
