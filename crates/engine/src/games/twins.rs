//! Join every pair of colored dots so that the lines fill the whole board.

use crate::board::Board;
use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "twins",
    title: "Twin Trace",
    brief: "Pairs of colored dots sit on a six by six board. Press on a dot and drag \
            through the squares to draw a line to the other dot of the same color. Lines go \
            up, down, left, and right, and cannot cross or pass through another color. Drag \
            back along a line to shorten it, or press on a dot to start its line over. A \
            board is done when every pair is joined and every square is covered. There are \
            three boards.",
    controls: &[
        "press on a dot and drag to draw its line",
        "press on a line to carry on from there",
    ],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle, Tag::Grid, Tag::Drag],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Twins::new())
}

pub(crate) const SIDE: usize = 6;
const CELL: f32 = 64.0;
const BOARD: Board = Board::centered(SIDE, SIDE, CELL);

/// Each letter is one color's two dots.
pub(crate) const BOARDS: [[&str; SIDE]; 3] = [
    ["BA...F", ".....E", "C..AF.", "D...B.", "....C.", "DE...."],
    ["....B.", ".DCEC.", "......", "......", "...EDB", "A....A"],
    ["..CD..", "....F.", "C.ED..", "B.....", ".....E", "BA.A.F"],
];

const COLORS: [Rgb; 6] =
    [palette::BAD, palette::PLAYER, palette::GOOD, palette::WARN, palette::MARK, palette::TEAL];

/// The two squares each color's dots sit in, in the order the letters run.
pub(crate) fn dots(board: usize) -> Vec<[usize; 2]> {
    let letters: String = BOARDS[board].concat();
    let mut pairs = Vec::new();
    for letter in 'A'..='F' {
        let found: Vec<usize> = letters.match_indices(letter).map(|(at, _)| at).collect();
        if let [one, two] = found[..] {
            pairs.push([one, two]);
        }
    }
    pairs
}

pub(crate) fn middle(cell: usize) -> (f32, f32) {
    BOARD.middle(cell % SIDE, cell / SIDE)
}

pub(crate) fn under(at: (f32, f32)) -> Option<usize> {
    BOARD.under(at).map(|(column, row)| row * SIDE + column)
}

pub(crate) struct Twins {
    pub(crate) board: usize,
    dots: Vec<[usize; 2]>,
    /// Each color's line, from the dot it was started at.
    lines: Vec<Vec<usize>>,
    drawing: Option<usize>,
}

impl Twins {
    pub(crate) fn new() -> Self {
        let dots = dots(0);
        Twins { board: 0, lines: vec![Vec::new(); dots.len()], dots, drawing: None }
    }

    fn owner(&self, cell: usize) -> Option<usize> {
        (0..self.dots.len())
            .find(|color| self.dots[*color].contains(&cell) || self.lines[*color].contains(&cell))
    }

    fn joined(&self, color: usize) -> bool {
        let line = &self.lines[color];
        line.len() > 1
            && self.dots[color].contains(&line[0])
            && self.dots[color].contains(&line[line.len() - 1])
    }

    /// Carry a line one square at a time toward the pointer, as long as the
    /// pointer is straight along a row or a column from the end of it.
    fn extend(&mut self, color: usize, toward: usize) {
        while let Some(&head) = self.lines[color].last() {
            if head == toward || (head % SIDE != toward % SIDE && head / SIDE != toward / SIDE) {
                return;
            }
            let next = match (toward % SIDE).cmp(&(head % SIDE)) {
                std::cmp::Ordering::Greater => head + 1,
                std::cmp::Ordering::Less => head - 1,
                std::cmp::Ordering::Equal if toward > head => head + SIDE,
                std::cmp::Ordering::Equal => head - SIDE,
            };
            if let Some(back) = self.lines[color].iter().position(|cell| *cell == next) {
                self.lines[color].truncate(back + 1);
            } else if self.joined(color) || self.owner(next).is_some_and(|other| other != color) {
                return;
            } else {
                self.lines[color].push(next);
            }
        }
    }
}

impl Game for Twins {
    fn update(&mut self, input: &Input) -> Outcome {
        let cell = under(input.cursor());
        if input.button_pressed(Button::Left) {
            self.drawing = cell.and_then(|cell| self.owner(cell));
            if let (Some(color), Some(cell)) = (self.drawing, cell) {
                let line = &mut self.lines[color];
                match line.iter().position(|at| *at == cell) {
                    Some(at) => line.truncate(at + 1),
                    None => *line = vec![cell],
                }
            }
        }
        if !input.button_held(Button::Left) {
            self.drawing = None;
        }
        if let (Some(color), Some(cell)) = (self.drawing, cell) {
            self.extend(color, cell);
        }

        let covered = (0..SIDE * SIDE).all(|cell| self.owner(cell).is_some());
        if !covered || !(0..self.dots.len()).all(|color| self.joined(color)) {
            return Outcome::Playing;
        }
        if self.board + 1 == BOARDS.len() {
            return Outcome::Won;
        }
        self.board += 1;
        self.dots = dots(self.board);
        self.lines = vec![Vec::new(); self.dots.len()];
        self.drawing = None;
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let (across, down) = BOARD.span();
        frame.rect(BOARD.left, BOARD.top, across, down, palette::PANEL);
        for (color, line) in self.lines.iter().enumerate() {
            let tint =
                COLORS[color].mix(palette::PANEL, if self.joined(color) { 0.55 } else { 0.8 });
            for cell in line {
                let (x, y) = BOARD.corner(cell % SIDE, cell / SIDE);
                frame.rect(x, y, CELL, CELL, tint);
            }
        }
        for step in 0..=SIDE {
            let along = step as f32 * CELL;
            frame.rect(BOARD.left + along - 1.0, BOARD.top, 2.0, down, palette::EDGE);
            frame.rect(BOARD.left, BOARD.top + along - 1.0, across, 2.0, palette::EDGE);
        }
        for (color, line) in self.lines.iter().enumerate() {
            for pair in line.windows(2) {
                let (a, b) = (middle(pair[0]), middle(pair[1]));
                frame.line(a.0, a.1, b.0, b.1, 16.0, COLORS[color]);
            }
            for cell in line {
                let at = middle(*cell);
                frame.disc(at.0, at.1, 8.0, COLORS[color]);
            }
        }
        for (color, pair) in self.dots.iter().enumerate() {
            for cell in pair {
                let at = middle(*cell);
                frame.disc(at.0, at.1, 22.0, COLORS[color]);
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/twins.rs"]
mod tests;
