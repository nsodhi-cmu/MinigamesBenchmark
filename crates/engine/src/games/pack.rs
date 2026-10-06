//! Fill a grid with a handful of pieces, each used once.

use crate::board::Board;
use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "pack",
    title: "Pieces Packing",
    brief: "A six by six grid has a few squares already filled in. Fill every other square \
            with the pieces beside it, using each piece once. Press on a piece and drag it \
            onto the grid: the square you hold it by lands on the square you let go over. \
            Right click a piece to turn it a quarter turn clockwise about the square you \
            click, which also works while dragging. Pieces cannot be flipped over. A piece \
            let go where it does not fit, or turned where it no longer fits, goes back beside \
            the grid. There are three grids.",
    controls: &[
        "press on a piece and drag it onto the grid",
        "right click a piece to turn it clockwise",
    ],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle, Tag::Grid, Tag::Drag],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Pack::new())
}

pub(crate) const SIDE: usize = 6;
pub(crate) const CELL: f32 = 48.0;
pub(crate) const BOARD: Board =
    Board::at(36.0, (frame::HEIGHT as f32 - CELL * SIDE as f32) * 0.5, SIDE, SIDE, CELL);
const SMALL: f32 = 22.0;
const TRAY_LEFT: f32 = 356.0;
const SLOT_WIDE: f32 = 136.0;
const SLOT_TALL: f32 = 108.0;
/// The tray is three slots deep, centered like the grid beside it.
const TRAY_TOP: f32 = (frame::HEIGHT as f32 - 3.0 * SLOT_TALL) * 0.5;

/// Each grid's one answer: `#` a square filled from the start, each letter a piece.
pub(crate) const BOARDS: [[&str; SIDE]; 3] = [
    ["#AA#D#", "AADDD#", "#AB#D#", "CCBBBB", "CCEEE#", "C#E#E#"],
    ["DDDAEE", "#DDAE#", "FFAAEE", "#FF#B#", "C#FBBB", "CCCC#B"],
    ["CCCDDD", "#C#D#D", "FCBBAA", "F#BAAE", "FFB#AE", "F#BEEE"],
];
/// The quarter turns clockwise each piece starts off from its place in the answer.
pub(crate) const TURNS: [&[u8]; 3] = [&[2, 1, 3, 1, 2], &[1, 3, 2, 0, 3, 1], &[3, 2, 1, 3, 1, 2]];

const TINTS: [Rgb; 8] = [
    palette::PLAYER,
    palette::GOOD,
    palette::WARN,
    palette::BAD,
    palette::MARK,
    palette::PINK,
    palette::TEAL,
    palette::ORANGE,
];

/// Squares of a piece as rows and columns from the corner of the box around it.
pub(crate) type Shape = Vec<(i32, i32)>;

/// A quarter turn clockwise, keeping each square where it was in the list.
pub(crate) fn turned(shape: &[(i32, i32)]) -> Shape {
    let spun: Shape = shape.iter().map(|(row, column)| (*column, -*row)).collect();
    let top = spun.iter().map(|square| square.0).min().unwrap_or(0);
    let left = spun.iter().map(|square| square.1).min().unwrap_or(0);
    spun.iter().map(|(row, column)| (row - top, column - left)).collect()
}

/// The squares filled from the start, and each piece as it is in the answer.
pub(crate) fn parts(board: usize) -> (u64, Vec<Shape>) {
    let letters: Vec<char> = BOARDS[board].concat().chars().collect();
    let filled = (0..SIDE * SIDE).filter(|cell| letters[*cell] == '#').fold(0, |m, c| m | 1 << c);
    let mut names: Vec<char> = letters.iter().copied().filter(|c| *c != '#').collect();
    names.sort_unstable();
    names.dedup();
    let shapes = names
        .into_iter()
        .map(|name| {
            let cells: Vec<usize> = (0..SIDE * SIDE).filter(|c| letters[*c] == name).collect();
            let top = cells.iter().map(|c| c / SIDE).min().unwrap();
            let left = cells.iter().map(|c| c % SIDE).min().unwrap();
            cells.iter().map(|c| ((c / SIDE - top) as i32, (c % SIDE - left) as i32)).collect()
        })
        .collect();
    (filled, shapes)
}

pub(crate) struct Piece {
    pub(crate) shape: Shape,
    /// Where the corner of its box sits on the grid, if it is on the grid.
    pub(crate) at: Option<(i32, i32)>,
}

struct Held {
    piece: usize,
    /// Which of its squares the pointer holds it by.
    grip: usize,
}

pub(crate) struct Pack {
    pub(crate) board: usize,
    filled: u64,
    pub(crate) pieces: Vec<Piece>,
    held: Option<Held>,
    pointer: (f32, f32),
}

impl Pack {
    pub(crate) fn new() -> Self {
        let mut pack =
            Pack { board: 0, filled: 0, pieces: Vec::new(), held: None, pointer: (0.0, 0.0) };
        pack.lay(0);
        pack
    }

    fn lay(&mut self, board: usize) {
        let (filled, shapes) = parts(board);
        self.board = board;
        self.filled = filled;
        self.pieces = shapes
            .into_iter()
            .zip(TURNS[board])
            .map(|(mut shape, turns)| {
                for _ in 0..*turns {
                    shape = turned(&shape);
                }
                Piece { shape, at: None }
            })
            .collect();
    }

    /// Whether `shape` with its corner at `at` lies on open squares of the grid,
    /// clear of every piece but `which`.
    fn fits(&self, which: usize, shape: &[(i32, i32)], at: (i32, i32)) -> bool {
        shape.iter().all(|(row, column)| {
            let (row, column) = (at.0 + row, at.1 + column);
            let on = (0..SIDE as i32).contains(&row) && (0..SIDE as i32).contains(&column);
            on && self.filled >> (row as usize * SIDE + column as usize) & 1 == 0
                && self.pieces.iter().enumerate().all(|(other, piece)| {
                    other == which
                        || piece
                            .at
                            .is_none_or(|(r, c)| !piece.shape.contains(&(row - r, column - c)))
                })
        })
    }

    /// The piece and the square of it under the pointer, on the grid or beside it.
    fn under(&self, at: (f32, f32)) -> Option<(usize, usize)> {
        let square = square_under(at);
        self.pieces.iter().enumerate().find_map(|(which, piece)| {
            let grip = match (piece.at, square) {
                (Some(corner), Some(square)) => {
                    piece.shape.iter().position(|s| (corner.0 + s.0, corner.1 + s.1) == square)
                }
                (None, _) => {
                    let (x, y) = tray_corner(which, &piece.shape);
                    piece.shape.iter().position(|(row, column)| {
                        let (left, top) = (x + *column as f32 * SMALL, y + *row as f32 * SMALL);
                        (left..left + SMALL).contains(&at.0) && (top..top + SMALL).contains(&at.1)
                    })
                }
                _ => None,
            };
            grip.map(|grip| (which, grip))
        })
    }

    fn turn(&mut self, which: usize, grip: usize) {
        let piece = &self.pieces[which];
        let shape = turned(&piece.shape);
        let at = piece.at.map(|corner| {
            let (old, new) = (piece.shape[grip], shape[grip]);
            (corner.0 + old.0 - new.0, corner.1 + old.1 - new.1)
        });
        let at = at.filter(|at| self.fits(which, &shape, *at));
        self.pieces[which] = Piece { shape, at };
    }
}

pub(crate) fn square_under(at: (f32, f32)) -> Option<(i32, i32)> {
    BOARD.under(at).map(|(column, row)| (row as i32, column as i32))
}

/// The top left of a piece's box where it waits beside the grid.
pub(crate) fn tray_corner(which: usize, shape: &[(i32, i32)]) -> (f32, f32) {
    let rows = shape.iter().map(|s| s.0).max().unwrap_or(0) + 1;
    let columns = shape.iter().map(|s| s.1).max().unwrap_or(0) + 1;
    let (slot_x, slot_y) =
        (TRAY_LEFT + (which % 2) as f32 * SLOT_WIDE, TRAY_TOP + (which / 2) as f32 * SLOT_TALL);
    (
        slot_x + (SLOT_WIDE - columns as f32 * SMALL) * 0.5,
        slot_y + (SLOT_TALL - rows as f32 * SMALL) * 0.5,
    )
}

/// A piece's squares drawn from `corner` at `size` a square, with a dark edge
/// wherever a square has no square of the same piece beside it.
fn draw_shape(frame: &mut Frame, shape: &[(i32, i32)], corner: (f32, f32), size: f32, tint: Rgb) {
    for (row, column) in shape {
        let (x, y) = (corner.0 + *column as f32 * size, corner.1 + *row as f32 * size);
        frame.rect(x, y, size, size, tint);
        let edge = |dr: i32, dc: i32| !shape.contains(&(row + dr, column + dc));
        let weight = (size / 16.0).max(2.0);
        if edge(-1, 0) {
            frame.rect(x, y, size, weight, palette::BACKDROP);
        }
        if edge(1, 0) {
            frame.rect(x, y + size - weight, size, weight, palette::BACKDROP);
        }
        if edge(0, -1) {
            frame.rect(x, y, weight, size, palette::BACKDROP);
        }
        if edge(0, 1) {
            frame.rect(x + size - weight, y, weight, size, palette::BACKDROP);
        }
    }
}

impl Game for Pack {
    fn update(&mut self, input: &Input) -> Outcome {
        let pointer = input.cursor();
        self.pointer = pointer;
        if input.button_pressed(Button::Left)
            && self.held.is_none()
            && let Some((piece, grip)) = self.under(pointer)
        {
            self.pieces[piece].at = None;
            self.held = Some(Held { piece, grip });
        }
        if input.button_pressed(Button::Right) {
            let target = match &self.held {
                Some(held) => Some((held.piece, held.grip)),
                None => self.under(pointer),
            };
            if let Some((piece, grip)) = target {
                self.turn(piece, grip);
            }
        }
        if !input.button_held(Button::Left)
            && let Some(held) = self.held.take()
        {
            let shape = &self.pieces[held.piece].shape;
            let grip = shape[held.grip];
            let at = square_under(pointer).map(|(row, column)| (row - grip.0, column - grip.1));
            self.pieces[held.piece].at = at.filter(|at| self.fits(held.piece, shape, *at));
        }

        if self.held.is_some() || self.pieces.iter().any(|piece| piece.at.is_none()) {
            return Outcome::Playing;
        }
        if self.board + 1 == BOARDS.len() {
            return Outcome::Won;
        }
        self.lay(self.board + 1);
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let (across, down) = BOARD.span();
        frame.rect(BOARD.left - 4.0, BOARD.top - 4.0, across + 8.0, down + 8.0, palette::EDGE);
        for cell in 0..SIDE * SIDE {
            let (x, y) = BOARD.corner(cell % SIDE, cell / SIDE);
            if self.filled >> cell & 1 == 1 {
                frame.rect(x, y, CELL, CELL, palette::MUTED);
                frame.cross(x + CELL * 0.5, y + CELL * 0.5, CELL * 0.5 - 10.0, 4.0, palette::EDGE);
            } else {
                frame.rect(x + 1.0, y + 1.0, CELL - 2.0, CELL - 2.0, palette::PANEL);
            }
        }

        for (which, piece) in self.pieces.iter().enumerate() {
            let tint = TINTS[which % TINTS.len()];
            let held = self.held.as_ref().is_some_and(|held| held.piece == which);
            match piece.at {
                _ if held => {}
                Some((row, column)) => {
                    let corner = BOARD.corner(column as usize, row as usize);
                    draw_shape(frame, &piece.shape, corner, CELL, tint);
                }
                None => {
                    draw_shape(frame, &piece.shape, tray_corner(which, &piece.shape), SMALL, tint)
                }
            }
        }
        if let Some(held) = &self.held {
            let piece = &self.pieces[held.piece];
            let grip = piece.shape[held.grip];
            let corner = (
                self.pointer.0 - (grip.1 as f32 + 0.5) * CELL,
                self.pointer.1 - (grip.0 as f32 + 0.5) * CELL,
            );
            draw_shape(frame, &piece.shape, corner, CELL, TINTS[held.piece % TINTS.len()]);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/pack.rs"]
mod tests;
