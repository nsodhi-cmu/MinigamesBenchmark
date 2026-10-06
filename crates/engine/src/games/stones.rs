//! Take stones from piles in turns against an opponent that never misses a win.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "stones",
    title: "Stone Standoff",
    brief: "Stones stand in piles, and you and an opponent take turns removing them. On your \
            turn, click a stone to take it and every stone above it in its pile: any number from \
            one pile, but at least one. The opponent answers at once, and the stones it took are \
            left ringed until you move again. Whoever takes the last \
            stone wins the round. You go first in every round, and every round starts from a \
            position you can win from, but the opponent never misses a winning move once you \
            give it one. There are three rounds.",
    controls: &["left click a stone to take it and the stones above it"],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Strict, Tag::Puzzle],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Stones::new())
}

/// Each round's piles from left to right.
pub(crate) const ROUNDS: [&[u8]; 3] = [&[5, 3, 4], &[5, 9, 3], &[6, 3, 8, 4]];
const STONE: f32 = 16.0;
const STEP: f32 = 36.0;
const SPACING: f32 = 120.0;
const FLOOR: f32 = 420.0;

/// The piles after the opponent's turn from `piles`: whatever leaves the
/// piles' counts canceling bit by bit if anything does, and otherwise one
/// stone off the largest pile.
pub(crate) fn reply(piles: &[u8]) -> Vec<u8> {
    let mut out = piles.to_vec();
    let sum = piles.iter().fold(0, |sum, pile| sum ^ pile);
    if let Some(at) = piles.iter().position(|pile| pile ^ sum < *pile) {
        out[at] ^= sum;
    } else if let Some(at) = (0..piles.len()).rev().max_by_key(|at| piles[*at]) {
        out[at] = out[at].saturating_sub(1);
    }
    out
}

/// The middle of the stone `height` up from the bottom of pile `pile` of `piles`.
pub(crate) fn center(piles: usize, pile: usize, height: usize) -> (f32, f32) {
    let left = (frame::WIDTH as f32 - SPACING * (piles - 1) as f32) * 0.5;
    (left + SPACING * pile as f32, FLOOR - STEP * height as f32)
}

pub(crate) struct Stones {
    pub(crate) round: usize,
    pub(crate) piles: Vec<u8>,
    /// The stones the opponent took last, as a pile and the heights it took from and to.
    taken: Option<(usize, u8, u8)>,
    cursor: (f32, f32),
}

impl Stones {
    pub(crate) fn new() -> Self {
        Stones { round: 0, piles: ROUNDS[0].to_vec(), taken: None, cursor: (0.0, 0.0) }
    }

    /// The pile and height of the stone under `at`, if there is one.
    fn under(&self, at: (f32, f32)) -> Option<(usize, u8)> {
        (0..self.piles.len()).find_map(|pile| {
            (0..self.piles[pile]).find_map(|height| {
                let (x, y) = center(self.piles.len(), pile, height as usize);
                let near = (x - at.0).abs() <= STEP * 0.5 && (y - at.1).abs() <= STEP * 0.5;
                near.then_some((pile, height))
            })
        })
    }
}

impl Game for Stones {
    fn update(&mut self, input: &Input) -> Outcome {
        self.cursor = input.cursor();
        if !input.button_pressed(Button::Left) {
            return Outcome::Playing;
        }
        let Some((pile, height)) = self.under(input.cursor()) else {
            return Outcome::Playing;
        };
        self.piles[pile] = height;
        self.taken = None;
        if self.piles.iter().all(|pile| *pile == 0) {
            if self.round + 1 == ROUNDS.len() {
                return Outcome::Won;
            }
            self.round += 1;
            self.piles = ROUNDS[self.round].to_vec();
            return Outcome::Playing;
        }

        let after = reply(&self.piles);
        let took = (0..after.len()).find(|at| after[*at] != self.piles[*at]);
        self.taken = took.map(|at| (at, after[at], self.piles[at]));
        self.piles = after;
        if self.piles.iter().all(|pile| *pile == 0) { Outcome::Lost } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let middle = frame::WIDTH as f32 * 0.5;
        frame.text_center(middle, 24.0, "YOUR TURN", 3, palette::PLAYER);

        let aimed = self.under(self.cursor);
        let count = self.piles.len();
        for (pile, stones) in self.piles.iter().enumerate() {
            let (x, _) = center(count, pile, 0);
            frame.rect(x - STEP * 0.6, FLOOR + STONE + 4.0, STEP * 1.2, 3.0, palette::EDGE);
            frame.text_center(x, FLOOR + STONE + 16.0, &stones.to_string(), 2, palette::MUTED);
            for height in 0..*stones {
                let (x, y) = center(count, pile, height as usize);
                let doomed = aimed.is_some_and(|(p, h)| p == pile && height >= h);
                let tint = if doomed { palette::WARN } else { palette::INK };
                frame.disc(x, y, STONE, tint);
            }
        }
        if let Some((pile, from, to)) = self.taken {
            for height in from..to {
                let (x, y) = center(count, pile, height as usize);
                frame.ring(x, y, STONE - 1.0, 3.0, palette::PINK);
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/stones.rs"]
mod tests;
