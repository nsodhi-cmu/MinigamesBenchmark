//! Fill the squares the numbers describe.

use crate::board::Board;
use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "glyph",
    title: "Grid Glyph",
    brief: "The numbers beside each row and above each column say how long the runs of \
            filled squares are, in order, with a gap between them. Fill the squares that \
            fit every number. Left click fills a square, right click marks one you believe \
            is blank. The grid finishes itself when it is right.",
    controls: &["left click fills a square", "right click marks one blank"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle, Tag::Grid],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Glyph::new(seed))
}

pub(crate) const SIDE: usize = 5;
const CELL: f32 = 64.0;
/// Room above the grid for the numbers over the columns, which are set at `WRIT`.
const CLUE_TOP: f32 = 72.0;
const WRIT: u32 = 2;
/// The drop from one line of numbers above a column to the next.
const LINE: f32 = 20.0;

pub(crate) type Lines = [u8; SIDE];

/// The squares of one line, as bits. Only the low `SIDE` of the byte are a line.
const WHOLE: u8 = ((1u16 << SIDE) - 1) as u8;

/// Every way the runs in `clue` can sit in a line of `SIDE` squares.
pub(crate) fn placements(clue: &[usize]) -> Vec<u8> {
    fn walk(clue: &[usize], from: usize, laid: u8, out: &mut Vec<u8>) {
        let Some((run, rest)) = clue.split_first() else {
            out.push(laid);
            return;
        };
        let needed: usize = rest.iter().sum::<usize>() + rest.len();
        let mut start = from;
        while start + run + needed <= SIDE {
            let mut mask = laid;
            for cell in start..start + run {
                mask |= 1 << cell;
            }
            walk(rest, start + run + 1, mask, out);
            start += 1;
        }
    }
    let mut out = Vec::new();
    walk(clue, 0, 0, &mut out);
    out
}

pub(crate) fn runs(mask: u8) -> Vec<usize> {
    let mut out = Vec::new();
    let mut run = 0;
    for cell in 0..SIDE {
        if mask >> cell & 1 == 1 {
            run += 1;
        } else if run > 0 {
            out.push(run);
            run = 0;
        }
    }
    if run > 0 {
        out.push(run);
    }
    out
}

fn column_of(rows: &Lines, column: usize) -> u8 {
    (0..SIDE).fold(0u8, |mask, row| mask | (((rows[row] >> column) & 1) << row))
}

/// Whether the puzzle falls out of looking at one line at a time, with nothing
/// ever guessed. A puzzle that does is both solvable and solvable exactly once.
pub(crate) fn reasons_out(down: &[Vec<usize>], across: &[Vec<usize>]) -> bool {
    let (mut filled, mut blank): (Lines, Lines) = ([0; SIDE], [0; SIDE]);
    loop {
        let before = (filled, blank);
        for line in 0..SIDE * 2 {
            let (index, clue) =
                if line < SIDE { (line, &down[line]) } else { (line - SIDE, &across[line - SIDE]) };
            let (known, void) = if line < SIDE {
                (filled[index], blank[index])
            } else {
                (column_of(&filled, index), column_of(&blank, index))
            };
            let fits: Vec<u8> = placements(clue)
                .into_iter()
                .filter(|mask| mask & void == 0 && known & !mask == 0)
                .collect();
            if fits.is_empty() {
                return false;
            }
            let sure = fits.iter().fold(WHOLE, |all, mask| all & mask);
            let never = !fits.iter().fold(0u8, |any, mask| any | mask) & WHOLE;
            for cell in 0..SIDE {
                let (row, column) = if line < SIDE { (index, cell) } else { (cell, index) };
                if sure >> cell & 1 == 1 {
                    filled[row] |= 1 << column;
                }
                if never >> cell & 1 == 1 {
                    blank[row] |= 1 << column;
                }
            }
        }
        if (filled, blank) == before {
            return (0..SIDE).all(|row| filled[row] | blank[row] == WHOLE);
        }
    }
}

/// A picture whose numbers can be reasoned all the way out.
pub(crate) fn draw_puzzle(rng: &mut Rng) -> (Lines, Vec<Vec<usize>>, Vec<Vec<usize>>) {
    loop {
        let mut answer: Lines = [0; SIDE];
        for row in answer.iter_mut() {
            for cell in 0..SIDE {
                if rng.chance(0.55) {
                    *row |= 1 << cell;
                }
            }
        }
        let down: Vec<Vec<usize>> = answer.iter().map(|mask| runs(*mask)).collect();
        let across: Vec<Vec<usize>> =
            (0..SIDE).map(|column| runs(column_of(&answer, column))).collect();
        // An empty picture reasons out and is already finished the moment it
        // is dealt, so it is no picture to be given.
        if answer.iter().any(|row| *row != 0) && reasons_out(&down, &across) {
            return (answer, down, across);
        }
    }
}

struct Glyph {
    answer: Lines,
    down: Vec<Vec<usize>>,
    across: Vec<Vec<usize>>,
    filled: Lines,
    blank: Lines,
}

impl Glyph {
    fn new(seed: u64) -> Self {
        let (answer, down, across) = draw_puzzle(&mut Rng::new(seed));
        Glyph { answer, down, across, filled: [0; SIDE], blank: [0; SIDE] }
    }

    /// The grid's corner: centered on the screen where it fits, and pushed down
    /// past the numbers along the top where it does not.
    fn board() -> Board {
        let span = SIDE as f32 * CELL;
        let centered = (frame::HEIGHT as f32 - span) * 0.5;
        let top = centered.max(CLUE_TOP);
        Board::at(frame::WIDTH as f32 * 0.5 - span * 0.5, top, SIDE, SIDE, CELL)
    }
}

impl Game for Glyph {
    fn update(&mut self, input: &Input) -> Outcome {
        let left = input.button_pressed(Button::Left);
        let right = input.button_pressed(Button::Right);
        if (left || right)
            && let Some((column, row)) = Glyph::board().under(input.cursor())
        {
            let bit = 1u8 << column;
            if left {
                self.blank[row] &= !bit;
                self.filled[row] ^= bit;
            } else {
                self.filled[row] &= !bit;
                self.blank[row] ^= bit;
            }
        }
        if self.filled == self.answer { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let board = Glyph::board();
        let (ox, oy) = (board.left, board.top);
        let (across, down) = board.span();
        frame.rect(ox, oy, across, down, palette::PANEL);

        for row in 0..SIDE {
            for column in 0..SIDE {
                let (x, y) = board.corner(column, row);
                if self.filled[row] >> column & 1 == 1 {
                    frame.rect(x + 1.0, y + 1.0, CELL - 2.0, CELL - 2.0, palette::INK);
                } else if self.blank[row] >> column & 1 == 1 {
                    frame.disc(x + CELL * 0.5, y + CELL * 0.5, 4.0, palette::EDGE);
                }
            }
        }
        for step in 0..=SIDE {
            let heavy = step == 0 || step == SIDE;
            let weight = if heavy { 3.0 } else { 1.0 };
            let tint = if heavy { palette::MUTED } else { palette::EDGE };
            let along = step as f32 * CELL;
            frame.rect(ox + along - weight * 0.5, oy, weight, down, tint);
            frame.rect(ox, oy + along - weight * 0.5, across, weight, tint);
        }

        for (row, clue) in self.down.iter().enumerate() {
            let text: String = if clue.is_empty() {
                "0".into()
            } else {
                clue.iter().map(|run| run.to_string()).collect::<Vec<_>>().join(" ")
            };
            let width = frame::text_width(&text, WRIT);
            frame.text(
                ox - 10.0 - width,
                oy + row as f32 * CELL + (CELL - frame::text_height(WRIT)) * 0.5,
                &text,
                WRIT,
                palette::INK,
            );
        }
        for (column, clue) in self.across.iter().enumerate() {
            let shown: Vec<String> = if clue.is_empty() {
                vec!["0".into()]
            } else {
                clue.iter().map(|run| run.to_string()).collect()
            };
            for (step, run) in shown.iter().enumerate() {
                let y = oy - 6.0 - (shown.len() - step) as f32 * LINE;
                frame.text_center(
                    ox + column as f32 * CELL + CELL * 0.5,
                    y,
                    run,
                    WRIT,
                    palette::INK,
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/glyph.rs"]
mod tests;
