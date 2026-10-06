//! Steer a growing line onto pellets without folding it into itself.

use crate::board::Board;
use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::Input;
use crate::palette;
use crate::rng::Rng;
use std::collections::VecDeque;

pub const SPEC: Spec = Spec {
    name: "serpent",
    title: "Super Serpent",
    brief: "A line crawls across the grid and never stops. Steer it onto the pale pellets; \
            each one you swallow makes it longer. Ten pellets finish the round. Touch a \
            wall or any part of the line itself and the round is over.",
    controls: &["arrow keys steer", "the line cannot turn back on itself"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Lethal, Tag::Navigate, Tag::Overhead, Tag::Grid],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Serpent::new(seed))
}

const COLUMNS: i32 = 26;
const ROWS: i32 = 18;
const CELL: f32 = 24.0;
const BOARD: Board = Board::at(
    8.0,
    (frame::HEIGHT as f32 - ROWS as f32 * CELL) * 0.5,
    COLUMNS as usize,
    ROWS as usize,
    CELL,
);
const STRIDE: u32 = 9;
const PELLETS: u32 = 10;

struct Serpent {
    rng: Rng,
    body: VecDeque<(i32, i32)>,
    heading: (i32, i32),
    turn: (i32, i32),
    pellet: (i32, i32),
    grow: u32,
    eaten: u32,
    countdown: u32,
}

impl Serpent {
    fn new(seed: u64) -> Self {
        let mut body = VecDeque::new();
        for offset in 0..4 {
            body.push_back((COLUMNS / 2 - offset, ROWS / 2));
        }
        let mut serpent = Serpent {
            rng: Rng::new(seed),
            body,
            heading: (1, 0),
            turn: (1, 0),
            pellet: (0, 0),
            grow: 0,
            eaten: 0,
            countdown: STRIDE,
        };
        serpent.scatter();
        serpent
    }

    /// A pellet on one of the squares the line is not on, found by counting
    /// off a random number of free squares.
    fn scatter(&mut self) {
        let free = (COLUMNS * ROWS) as usize - self.body.len();
        let mut skip = self.rng.below(free);
        for row in 0..ROWS {
            for column in 0..COLUMNS {
                let spot = (column, row);
                if self.body.contains(&spot) {
                    continue;
                }
                if skip == 0 {
                    self.pellet = spot;
                    return;
                }
                skip -= 1;
            }
        }
    }

    fn advance(&mut self) -> Outcome {
        self.heading = self.turn;
        let head = *self.body.front().expect("the line is never empty");
        let next = (head.0 + self.heading.0, head.1 + self.heading.1);
        if !BOARD.holds(next.0, next.1) {
            return Outcome::Lost;
        }
        if self.grow == 0 {
            self.body.pop_back();
        } else {
            self.grow -= 1;
        }
        if self.body.contains(&next) {
            return Outcome::Lost;
        }
        self.body.push_front(next);
        if next == self.pellet {
            self.eaten += 1;
            self.grow += 2;
            if self.eaten == PELLETS {
                return Outcome::Won;
            }
            self.scatter();
        }
        Outcome::Playing
    }
}

impl Game for Serpent {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some((_, direction)) = input.arrow()
            && direction != (-self.heading.0, -self.heading.1)
        {
            self.turn = direction;
        }
        self.countdown -= 1;
        if self.countdown == 0 {
            self.countdown = STRIDE;
            return self.advance();
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let (width, height) = BOARD.span();
        frame.rect(BOARD.left, BOARD.top, width, height, palette::PANEL);
        frame.outline(
            BOARD.left - 3.0,
            BOARD.top - 3.0,
            width + 6.0,
            height + 6.0,
            3.0,
            palette::EDGE,
        );

        let lines = palette::PANEL.mix(palette::EDGE, 0.55);
        for column in 1..COLUMNS {
            frame.rect(BOARD.left + column as f32 * CELL, BOARD.top, 1.0, height, lines);
        }
        for row in 1..ROWS {
            frame.rect(BOARD.left, BOARD.top + row as f32 * CELL, width, 1.0, lines);
        }

        let at = |cell: (i32, i32)| BOARD.corner(cell.0 as usize, cell.1 as usize);
        let (px, py) = at(self.pellet);
        frame.disc(px + CELL * 0.5, py + CELL * 0.5, CELL * 0.3, palette::INK);

        for (index, cell) in self.body.iter().enumerate() {
            let (x, y) = at(*cell);
            let tint = if index == 0 { palette::INK } else { palette::PLAYER };
            frame.rect(x + 2.0, y + 2.0, CELL - 4.0, CELL - 4.0, tint);
        }
    }
}
