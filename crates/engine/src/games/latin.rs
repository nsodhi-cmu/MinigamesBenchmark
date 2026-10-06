//! Fill a six by six grid so that no row and no column repeats a number.

use crate::board::Board;
use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "latin",
    title: "Latin Square",
    brief: "A six by six grid, part of it filled in already in gray. Fill in the rest so that \
            every row and every column holds each of the numbers one to six exactly once. The \
            diagonals do not matter. Click a square to pick it, then press a number key to \
            write that number in it, or BACKSPACE to rub it out. The gray numbers cannot be \
            changed. The grid is checked once every square has a number in it.",
    controls: &[
        "left click a square to pick it",
        "1 to 6 write a number",
        "BACKSPACE rubs one out",
    ],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle, Tag::Grid],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Latin::new(seed))
}

pub(crate) const SIDE: usize = 6;
pub(crate) const CELLS: usize = SIDE * SIDE;
pub(crate) const CELL: f32 = 58.0;
pub(crate) const BOARD: Board = Board::centered(SIDE, SIDE, CELL);

const RULE: Rgb = Rgb(150, 156, 172);
const WRITTEN: Rgb = Rgb(38, 110, 214);
const PICKED: Rgb = Rgb(206, 226, 250);

/// A grid, one number a square from one to six, or zero for an empty square.
pub(crate) type Grid = [u8; CELLS];

/// The numbers a square could take without repeating one in its row or column.
fn open(grid: &Grid, cell: usize) -> Vec<u8> {
    let (row, column) = (cell / SIDE, cell % SIDE);
    (1..=SIDE as u8)
        .filter(|number| {
            !(0..SIDE)
                .any(|at| grid[row * SIDE + at] == *number || grid[at * SIDE + column] == *number)
        })
        .collect()
}

/// The grid filled in as far as the squares that have only one number left and
/// the numbers that have only one square left in a row or column will take it.
pub(crate) fn reasoned(mut grid: Grid) -> Grid {
    loop {
        let mut wrote = false;
        for cell in 0..CELLS {
            if grid[cell] != 0 {
                continue;
            }
            if let [only] = open(&grid, cell)[..] {
                grid[cell] = only;
                wrote = true;
            }
        }
        for line in 0..SIDE {
            for number in 1..=SIDE as u8 {
                for cells in [
                    (0..SIDE).map(|at| line * SIDE + at).collect::<Vec<_>>(),
                    (0..SIDE).map(|at| at * SIDE + line).collect(),
                ] {
                    if cells.iter().any(|cell| grid[*cell] == number) {
                        continue;
                    }
                    let room: Vec<usize> = cells
                        .iter()
                        .copied()
                        .filter(|cell| grid[*cell] == 0 && open(&grid, *cell).contains(&number))
                        .collect();
                    if let [only] = room[..] {
                        grid[only] = number;
                        wrote = true;
                    }
                }
            }
        }
        if !wrote {
            return grid;
        }
    }
}

/// A whole grid with no row or column repeating a number, filled a square at a
/// time in a shuffled order of numbers and gone back over where a square runs out.
fn whole(rng: &mut Rng) -> Grid {
    let mut grid = [0; CELLS];
    let mut tried: Vec<Vec<u8>> = vec![Vec::new(); CELLS];
    let mut cell = 0;
    while cell < CELLS {
        let mut left: Vec<u8> =
            open(&grid, cell).into_iter().filter(|number| !tried[cell].contains(number)).collect();
        if left.is_empty() {
            tried[cell].clear();
            grid[cell] = 0;
            cell = cell.checked_sub(1).expect("the first square can take any number");
            grid[cell] = 0;
            continue;
        }
        rng.shuffle(&mut left);
        tried[cell].push(left[0]);
        grid[cell] = left[0];
        cell += 1;
    }
    grid
}

/// How many squares are left to fill in.
pub(crate) const BLANKS: usize = 18;

/// A grid and the squares given away in it: `BLANKS` of them rubbed out, and
/// only where the rest still follows without any guessing.
pub(crate) fn deal(rng: &mut Rng) -> (Grid, Grid) {
    let answer = whole(rng);
    let mut given = answer;
    let mut order: Vec<usize> = (0..CELLS).collect();
    rng.shuffle(&mut order);
    let mut blank = 0;
    for cell in order {
        if blank == BLANKS {
            break;
        }
        let was = given[cell];
        given[cell] = 0;
        if reasoned(given) == answer {
            blank += 1;
        } else {
            given[cell] = was;
        }
    }
    (answer, given)
}

pub(crate) struct Latin {
    pub(crate) given: Grid,
    pub(crate) grid: Grid,
    pub(crate) picked: Option<usize>,
}

impl Latin {
    pub(crate) fn new(seed: u64) -> Self {
        let (_, given) = deal(&mut Rng::new(seed));
        Latin { given, grid: given, picked: None }
    }
}

/// Whether every row and every column of a full grid holds each number once.
pub(crate) fn correct(grid: &Grid) -> bool {
    (0..SIDE).all(|line| {
        let mut seen = [(0u8, 0u8); SIDE];
        for at in 0..SIDE {
            let (row, column) = (grid[line * SIDE + at], grid[at * SIDE + line]);
            if row == 0 || column == 0 {
                return false;
            }
            seen[row as usize - 1].0 += 1;
            seen[column as usize - 1].1 += 1;
        }
        seen.iter().all(|(row, column)| *row == 1 && *column == 1)
    })
}

fn under(at: (f32, f32)) -> Option<usize> {
    BOARD.under(at).map(|(column, row)| row * SIDE + column)
}

const NUMBERS: [Key; SIDE] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6];

impl Game for Latin {
    fn update(&mut self, input: &Input) -> Outcome {
        if input.button_pressed(Button::Left)
            && let Some(cell) = under(input.cursor())
        {
            self.picked = Some(cell);
        }
        if let Some(cell) = self.picked.filter(|cell| self.given[*cell] == 0) {
            if input.pressed(Key::Backspace) {
                self.grid[cell] = 0;
            }
            if let Some(number) = NUMBERS.iter().position(|key| input.pressed(*key)) {
                self.grid[cell] = number as u8 + 1;
            }
        }
        // Asked whatever is picked out, so a finished grid is finished even
        // when what is picked is a square that was given.
        if correct(&self.grid) { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let (across, down) = BOARD.span();
        frame.rect(BOARD.left - 4.0, BOARD.top - 4.0, across + 8.0, down + 8.0, RULE);
        for cell in 0..CELLS {
            let (x, y) = BOARD.corner(cell % SIDE, cell / SIDE);
            let paper = if self.picked == Some(cell) { PICKED } else { palette::INK };
            frame.rect(x + 1.0, y + 1.0, CELL - 2.0, CELL - 2.0, paper);
            if self.grid[cell] == 0 {
                continue;
            }
            let tint = if self.given[cell] == 0 { WRITTEN } else { palette::EDGE };
            let number = self.grid[cell].to_string();
            frame.text_center(x + CELL * 0.5, y + CELL * 0.5 - 12.0, &number, 3, tint);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/latin.rs"]
mod tests;
