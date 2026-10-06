//! Turn every square black, where a click turns a square and its neighbors over.

use crate::board::Board;
use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "lights",
    title: "Last Lights",
    brief: "A six by six board of black and white squares. Clicking a square turns it over, \
            white to black or black to white, and turns over the squares directly above, \
            below, left, and right of it too. Turn every square black. There are three \
            boards.",
    controls: &["left click a square to turn it and its neighbors over"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle, Tag::Grid],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Lights::new(seed))
}

pub(crate) const SIDE: usize = 6;
pub(crate) const CELLS: usize = SIDE * SIDE;
/// How many squares each board was dealt by clicking.
pub(crate) const PRESSES: [usize; 3] = [3, 5, 7];
const CELL: f32 = 64.0;
const GAP: f32 = 4.0;
pub(crate) const BOARD: Board = Board::centered(SIDE, SIDE, CELL);

const BLACK: Rgb = Rgb(0, 0, 0);

/// The squares a click on `cell` turns over, one bit a square, row by row.
pub(crate) fn flips(cell: usize) -> u64 {
    let (row, column) = (cell / SIDE, cell % SIDE);
    let mut mask = 1u64 << cell;
    if row > 0 {
        mask |= 1 << (cell - SIDE);
    }
    if row + 1 < SIDE {
        mask |= 1 << (cell + SIDE);
    }
    if column > 0 {
        mask |= 1 << (cell - 1);
    }
    if column + 1 < SIDE {
        mask |= 1 << (cell + 1);
    }
    mask
}

/// A board made by clicking `presses` different squares of an all black one.
pub(crate) fn deal(rng: &mut Rng, presses: usize) -> u64 {
    let mut cells: Vec<usize> = (0..CELLS).collect();
    rng.shuffle(&mut cells);
    cells[..presses].iter().fold(0, |white, cell| white ^ flips(*cell))
}

pub(crate) struct Lights {
    rng: Rng,
    pub(crate) board: usize,
    /// The white squares, one bit a square.
    pub(crate) white: u64,
}

impl Lights {
    pub(crate) fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let white = deal(&mut rng, PRESSES[0]);
        Lights { rng, board: 0, white }
    }
}

pub(crate) fn middle(cell: usize) -> (f32, f32) {
    BOARD.middle(cell % SIDE, cell / SIDE)
}

fn under(at: (f32, f32)) -> Option<usize> {
    BOARD.under(at).map(|(column, row)| row * SIDE + column)
}

impl Game for Lights {
    fn update(&mut self, input: &Input) -> Outcome {
        if !input.button_pressed(Button::Left) {
            return Outcome::Playing;
        }
        let Some(cell) = under(input.cursor()) else { return Outcome::Playing };
        self.white ^= flips(cell);
        if self.white != 0 {
            return Outcome::Playing;
        }
        if self.board + 1 == PRESSES.len() {
            return Outcome::Won;
        }
        self.board += 1;
        self.white = deal(&mut self.rng, PRESSES[self.board]);
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let (across, down) = BOARD.span();
        frame.rect(
            BOARD.left - GAP,
            BOARD.top - GAP,
            across + GAP * 2.0,
            down + GAP * 2.0,
            palette::EDGE,
        );
        for cell in 0..CELLS {
            let (x, y) = middle(cell);
            let tint = if self.white >> cell & 1 == 1 { palette::INK } else { BLACK };
            let side = CELL - GAP * 2.0;
            frame.rect(x - side * 0.5, y - side * 0.5, side, side, tint);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/lights.rs"]
mod tests;
