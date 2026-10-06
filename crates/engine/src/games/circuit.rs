//! Drop three pieces of wire into the board and close the circuit.

use crate::board::Board;
use crate::frame::Frame;
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use std::collections::VecDeque;

pub const SPEC: Spec = Spec {
    name: "circuit",
    title: "Circuit Connect",
    brief: "Two terminals sit on opposite sides of the board with a run of wire between \
            them that does not quite meet, and some wire that goes nowhere at all. Three \
            sockets are open, each holding a right angle. Click one to turn it a quarter \
            turn, until every gap is closed and the current gets from one terminal to \
            the other. Five boards finish the round.",
    controls: &["left click a socket to turn the piece in it"],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle, Tag::Grid],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Circuit::new())
}

pub(crate) const LEVELS: usize = 5;
pub(crate) const COLUMNS: usize = 9;
pub(crate) const ROWS: usize = 5;
pub(crate) const PIECES: usize = 6;
/// The right angles a socket turns through, a quarter turn at a time.
pub(crate) const TURNS: [usize; 4] = [2, 5, 4, 3];
const CELL: f32 = 58.0;
const BOARD: Board = Board::centered(COLUMNS, ROWS, CELL);

/// Which sides a piece is open on: north, east, south, west.
const OPENINGS: [[bool; 4]; PIECES] = [
    [false, true, false, true], // straight across
    [true, false, true, false], // straight down
    [true, true, false, false], // corner north to east
    [true, false, false, true], // corner north to west
    [false, false, true, true], // corner south to west
    [false, true, true, false], // corner south to east
];

/// `.` bare board, `?` a socket, `A` and `B` the terminals, and the rest fixed wire.
pub(crate) const BOARDS: [[&str; ROWS]; LEVELS] = [
    ["...F-7...", "..?-7....", "A-?.?-7..", "......L-B", "..F-J...."],
    ["....L-7..", "A-?......", "..|.?-7..", "..?-J.L-B", "...F-7..."],
    ["....F-7..", "..F-7.F?.", "A-?.?-JLB", "...L-7...", "........."],
    ["..L-7....", "A-?.?-7..", "..|.|.|..", "..L-J.?-B", "....F-J.."],
    ["..F--?...", "..|..|...", "A-?..?--B", "...F-7...", "...L-J..."],
];

fn piece_of(mark: u8) -> Option<usize> {
    match mark {
        b'-' => Some(0),
        b'|' => Some(1),
        b'L' => Some(2),
        b'J' => Some(3),
        b'7' => Some(4),
        b'F' => Some(5),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Cell {
    Bare,
    Wire(usize),
    Socket(usize),
    Terminal(bool),
}

pub(crate) fn board_of(level: usize) -> Vec<Vec<Cell>> {
    BOARDS[level]
        .iter()
        .map(|line| {
            line.bytes()
                .map(|mark| match mark {
                    b'.' => Cell::Bare,
                    b'?' => Cell::Socket(0),
                    b'A' => Cell::Terminal(true),
                    b'B' => Cell::Terminal(false),
                    other => Cell::Wire(piece_of(other).expect("a known piece")),
                })
                .collect()
        })
        .collect()
}

/// Which sides a cell lets current out of. A terminal only faces the board.
fn faces(cell: Cell) -> [bool; 4] {
    match cell {
        Cell::Bare => [false; 4],
        Cell::Wire(piece) => OPENINGS[piece],
        Cell::Socket(turn) => OPENINGS[TURNS[turn]],
        Cell::Terminal(left) => {
            if left {
                [false, true, false, false]
            } else {
                [false, false, false, true]
            }
        }
    }
}

/// Whether current gets from one terminal to the other.
pub(crate) fn closed(board: &[Vec<Cell>]) -> bool {
    let find = |wanted: bool| {
        (0..ROWS)
            .flat_map(|row| (0..COLUMNS).map(move |column| (column, row)))
            .find(|(column, row)| board[*row][*column] == Cell::Terminal(wanted))
    };
    let (Some(from), Some(to)) = (find(true), find(false)) else {
        return false;
    };
    let mut seen = vec![vec![false; COLUMNS]; ROWS];
    let mut queue = VecDeque::from([from]);
    seen[from.1][from.0] = true;
    while let Some((column, row)) = queue.pop_front() {
        if (column, row) == to {
            return true;
        }
        let here = faces(board[row][column]);
        for (side, (dx, dy)) in [(0i32, -1i32), (1, 0), (0, 1), (-1, 0)].into_iter().enumerate() {
            if !here[side] {
                continue;
            }
            let (nx, ny) = (column as i32 + dx, row as i32 + dy);
            if nx < 0 || ny < 0 || nx >= COLUMNS as i32 || ny >= ROWS as i32 {
                continue;
            }
            let (nx, ny) = (nx as usize, ny as usize);
            if seen[ny][nx] {
                continue;
            }
            if faces(board[ny][nx])[(side + 2) % 4] {
                seen[ny][nx] = true;
                queue.push_back((nx, ny));
            }
        }
    }
    false
}

#[cfg(test)]
pub(crate) fn sockets_of(board: &[Vec<Cell>]) -> Vec<(usize, usize)> {
    (0..ROWS)
        .flat_map(|row| (0..COLUMNS).map(move |column| (column, row)))
        .filter(|(column, row)| matches!(board[*row][*column], Cell::Socket(_)))
        .collect()
}

struct Circuit {
    level: usize,
    board: Vec<Vec<Cell>>,
}

impl Circuit {
    fn new() -> Self {
        Circuit { level: 0, board: board_of(0) }
    }
}

impl Game for Circuit {
    fn update(&mut self, input: &Input) -> Outcome {
        if input.button_pressed(Button::Left)
            && let Some((column, row)) = BOARD.under(input.cursor())
            && let Cell::Socket(piece) = self.board[row][column]
        {
            self.board[row][column] = Cell::Socket((piece + 1) % TURNS.len());
        }
        if closed(&self.board) {
            if self.level + 1 == LEVELS {
                return Outcome::Won;
            }
            self.level += 1;
            self.board = board_of(self.level);
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let (across, down) = BOARD.span();
        frame.rect(BOARD.left - 8.0, BOARD.top - 8.0, across + 16.0, down + 16.0, palette::PANEL);
        for row in 0..ROWS {
            for column in 0..COLUMNS {
                let (x, y) = BOARD.corner(column, row);
                let (cx, cy) = BOARD.middle(column, row);
                let cell = self.board[row][column];
                frame.disc(cx, cy, 2.0, palette::EDGE);
                if let Cell::Socket(_) = cell {
                    frame.rect(x + 3.0, y + 3.0, CELL - 6.0, CELL - 6.0, palette::BACKDROP);
                    frame.outline(x + 3.0, y + 3.0, CELL - 6.0, CELL - 6.0, 2.0, palette::MARK);
                }
                if let Cell::Terminal(left) = cell {
                    let tint = if left { palette::WARN } else { palette::GOOD };
                    frame.disc(cx, cy, 17.0, tint);
                    frame.disc(cx, cy, 8.0, palette::BACKDROP);
                }
                let open = faces(cell);
                let tint = match cell {
                    Cell::Socket(_) => palette::MARK,
                    _ => palette::WARN,
                };
                for (side, (dx, dy)) in
                    [(0.0f32, -1.0f32), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)].into_iter().enumerate()
                {
                    if !open[side] || matches!(cell, Cell::Terminal(_)) {
                        continue;
                    }
                    frame.line(cx, cy, cx + dx * CELL * 0.5, cy + dy * CELL * 0.5, 7.0, tint);
                }
                if !matches!(cell, Cell::Bare | Cell::Terminal(_)) {
                    frame.disc(cx, cy, 5.0, tint);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/circuit.rs"]
mod tests;
