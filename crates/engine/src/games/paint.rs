//! Walk every square of a board once, where stepping on paint ends the round.

use crate::board::Board;
use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::Input;
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "paint",
    title: "Paint Path",
    brief: "You stand on a board of squares seen from above, and the square you stand on is \
            painted. Each arrow key steps you one square, painting it as you go. Walking into \
            the edge of the board does nothing. Paint every square of the board to finish. Step \
            on a square you have already painted and the round is over. Nothing warns you that \
            you have painted yourself into a corner, so plan the whole way around before you set \
            off. There are three boards.",
    controls: &["arrow keys step one square"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Strict, Tag::Puzzle, Tag::Overhead, Tag::Grid],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Paint::new())
}

pub(crate) const SIDE: usize = 11;
const CELL: f32 = 38.0;
const BOARD: Board = Board::centered(SIDE, SIDE, CELL);

const RIM: Rgb = Rgb(180, 184, 198);
const PAINT: Rgb = Rgb(86, 192, 232);

/// Each board a row at a time: `.` a square to paint, `S` where you start, and
/// `#` nothing at all.
pub(crate) const BOARDS: [[&str; SIDE]; 3] = [
    [
        "###########",
        "###########",
        "###S....###",
        "###.....###",
        "#####.#####",
        "#####.#####",
        "#####.#####",
        "###.....###",
        "###.....###",
        "###########",
        "###########",
    ],
    [
        "###########",
        "###########",
        "###..######",
        "###..######",
        "#....##..##",
        "#S........#",
        "##..##....#",
        "######..###",
        "######..###",
        "###########",
        "###########",
    ],
    [
        "###########",
        "###S..#####",
        "##....#####",
        "##..#..####",
        "###....####",
        "####...####",
        "####....###",
        "####..#..##",
        "#####....##",
        "#####...###",
        "###########",
    ],
];

pub(crate) fn square(board: usize, at: (i32, i32)) -> u8 {
    if !(0..SIDE as i32).contains(&at.0) || !(0..SIDE as i32).contains(&at.1) {
        return b'#';
    }
    BOARDS[board][at.1 as usize].as_bytes()[at.0 as usize]
}

/// Every square of a board that has to be painted.
pub(crate) fn squares(board: usize) -> Vec<(i32, i32)> {
    (0..SIDE as i32)
        .flat_map(|y| (0..SIDE as i32).map(move |x| (x, y)))
        .filter(|at| square(board, *at) != b'#')
        .collect()
}

pub(crate) fn start(board: usize) -> (i32, i32) {
    *squares(board).iter().find(|at| square(board, **at) == b'S').expect("a board has a start")
}

pub(crate) struct Paint {
    pub(crate) board: usize,
    pub(crate) at: (i32, i32),
    pub(crate) painted: Vec<(i32, i32)>,
}

impl Paint {
    pub(crate) fn new() -> Self {
        Paint::starting(0)
    }

    fn starting(board: usize) -> Self {
        let at = start(board);
        Paint { board, at, painted: vec![at] }
    }
}

impl Game for Paint {
    fn update(&mut self, input: &Input) -> Outcome {
        let Some((_, (dx, dy))) = input.arrow() else { return Outcome::Playing };

        let next = (self.at.0 + dx, self.at.1 + dy);
        if square(self.board, next) == b'#' {
            return Outcome::Playing;
        }
        if self.painted.contains(&next) {
            return Outcome::Lost;
        }
        self.at = next;
        self.painted.push(next);
        if self.painted.len() < squares(self.board).len() {
            return Outcome::Playing;
        }
        if self.board + 1 == BOARDS.len() {
            return Outcome::Won;
        }
        *self = Paint::starting(self.board + 1);
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        for at in squares(self.board) {
            let (x, y) = BOARD.corner(at.0 as usize, at.1 as usize);
            let tint = if self.painted.contains(&at) { PAINT } else { palette::INK };
            frame.bordered(x, y, CELL, CELL, 2.0, tint, RIM);
        }
        let (x, y) = BOARD.middle(self.at.0 as usize, self.at.1 as usize);
        frame.disc(x, y, CELL * 0.3, palette::BACKDROP);
        frame.disc(x, y, CELL * 0.18, palette::WARN);
    }
}

#[cfg(test)]
#[path = "../tests/games/paint.rs"]
mod tests;
