//! Open every square that is not a mine, and none that is.

use crate::board::Board;
use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "mines",
    title: "Mine Sweep",
    brief: "Ten mines are buried in the hundred squares. Click a square to open it: an open \
            square says how many of the eight squares around it hold a mine, and one with no \
            mine beside it opens its neighbors too. Open all ninety squares that are not \
            mines to finish. Open one that is and the round is over.",
    controls: &["left click a square to open it"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Strict, Tag::Puzzle, Tag::Explore, Tag::Grid],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Mines::new(seed))
}

pub(crate) const SIDE: usize = 10;
pub(crate) const MINES: usize = 10;
const CELL: f32 = 40.0;
const BOARD: Board = Board::centered(SIDE, SIDE, CELL);

pub(crate) type Field = [[bool; SIDE]; SIDE];
type Seen = [[bool; SIDE]; SIDE];

const COUNTS: [Rgb; 8] = [
    palette::PLAYER,
    palette::GOOD,
    palette::BAD,
    palette::PINK,
    palette::WARN,
    palette::MARK,
    palette::MUTED,
    palette::INK,
];

pub(crate) fn beside(column: usize, row: usize) -> impl Iterator<Item = (usize, usize)> {
    [(-1i32, -1i32), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)]
        .into_iter()
        .filter_map(move |(dc, dr)| {
            let (nc, nr) = (column as i32 + dc, row as i32 + dr);
            let inside = nc >= 0 && nr >= 0 && (nc as usize) < SIDE && (nr as usize) < SIDE;
            inside.then_some((nc as usize, nr as usize))
        })
}

pub(crate) fn count_at(mines: &Field, column: usize, row: usize) -> usize {
    beside(column, row).filter(|(nc, nr)| mines[*nr][*nc]).count()
}

/// Ten mines anywhere but the square first clicked and the ring around it, so
/// that the first click always opens a whole region.
fn lay(rng: &mut Rng, clear: (usize, usize)) -> Field {
    let mut spare: Vec<(usize, usize)> = (0..SIDE)
        .flat_map(|row| (0..SIDE).map(move |column| (column, row)))
        .filter(|(c, r)| {
            (*c, *r) != clear && !beside(clear.0, clear.1).any(|spot| spot == (*c, *r))
        })
        .collect();
    rng.shuffle(&mut spare);
    let mut mines = [[false; SIDE]; SIDE];
    for (column, row) in spare.iter().take(MINES) {
        mines[*row][*column] = true;
    }
    mines
}

/// Whether the field falls out one square at a time from the first click, with
/// nothing ever guessed at.
pub(crate) fn reasons_out(mines: &Field, first: (usize, usize)) -> bool {
    let mut open: Seen = [[false; SIDE]; SIDE];
    let mut known: Seen = [[false; SIDE]; SIDE];
    spread(mines, &mut open, first);
    loop {
        let before = (open, known);
        for row in 0..SIDE {
            for column in 0..SIDE {
                if !open[row][column] {
                    continue;
                }
                let says = count_at(mines, column, row);
                let shut: Vec<(usize, usize)> =
                    beside(column, row).filter(|(c, r)| !open[*r][*c]).collect();
                let flagged = shut.iter().filter(|(c, r)| known[*r][*c]).count();
                if flagged == says {
                    for (c, r) in shut.iter().filter(|(c, r)| !known[*r][*c]) {
                        spread(mines, &mut open, (*c, *r));
                    }
                } else if shut.len() == says {
                    for (c, r) in &shut {
                        known[*r][*c] = true;
                    }
                }
            }
        }
        pairs(mines, &mut open, &mut known);
        if (open, known) == before {
            return (0..SIDE)
                .all(|row| (0..SIDE).all(|column| open[row][column] || mines[row][column]));
        }
    }
}

/// The one step past counting: where the shut squares one number sees are all
/// among those another sees, the difference settles the rest.
fn pairs(mines: &Field, open: &mut Seen, known: &mut Seen) {
    let shut_of = |open: &Seen, c: usize, r: usize| -> Vec<(usize, usize)> {
        beside(c, r).filter(|(nc, nr)| !open[*nr][*nc]).collect()
    };
    for row in 0..SIDE {
        for column in 0..SIDE {
            if !open[row][column] {
                continue;
            }
            let mine = shut_of(open, column, row);
            let mine_left =
                count_at(mines, column, row) - mine.iter().filter(|(c, r)| known[*r][*c]).count();
            for (other_c, other_r) in beside(column, row).flat_map(|(c, r)| beside(c, r)) {
                if !open[other_r][other_c] || (other_c, other_r) == (column, row) {
                    continue;
                }
                let theirs = shut_of(open, other_c, other_r);
                if !mine.iter().all(|spot| theirs.contains(spot)) {
                    continue;
                }
                let theirs_left = count_at(mines, other_c, other_r)
                    - theirs.iter().filter(|(c, r)| known[*r][*c]).count();
                let extra: Vec<(usize, usize)> = theirs
                    .iter()
                    .filter(|spot| !mine.contains(spot) && !known[spot.1][spot.0])
                    .copied()
                    .collect();
                if theirs_left == mine_left {
                    for (c, r) in extra {
                        spread(mines, open, (c, r));
                    }
                } else if theirs_left - mine_left == extra.len() {
                    for (c, r) in extra {
                        known[r][c] = true;
                    }
                }
            }
        }
    }
}

/// Open a square, and where it has no mine beside it, everything around it too.
fn spread(mines: &Field, open: &mut Seen, at: (usize, usize)) {
    let mut edge = vec![at];
    while let Some((column, row)) = edge.pop() {
        if open[row][column] || mines[row][column] {
            continue;
        }
        open[row][column] = true;
        if count_at(mines, column, row) == 0 {
            edge.extend(beside(column, row));
        }
    }
}

struct Mines {
    rng: Rng,
    mines: Option<Field>,
    open: Seen,
    struck: Option<(usize, usize)>,
}

impl Mines {
    fn new(seed: u64) -> Self {
        Mines { rng: Rng::new(seed), mines: None, open: [[false; SIDE]; SIDE], struck: None }
    }

    fn opened(&self) -> usize {
        self.open.iter().flatten().filter(|seen| **seen).count()
    }
}

impl Game for Mines {
    fn update(&mut self, input: &Input) -> Outcome {
        if !input.button_pressed(Button::Left) {
            return Outcome::Playing;
        }
        let (x, y) = input.cursor();
        let Some((column, row)) = BOARD.under((x, y)) else {
            return Outcome::Playing;
        };
        if self.open[row][column] {
            return Outcome::Playing;
        }
        let mines = *self.mines.get_or_insert_with(|| {
            let mut laid = lay(&mut self.rng, (column, row));
            while !reasons_out(&laid, (column, row)) {
                laid = lay(&mut self.rng, (column, row));
            }
            laid
        });
        if mines[row][column] {
            self.struck = Some((column, row));
            return Outcome::Lost;
        }
        spread(&mines, &mut self.open, (column, row));
        if self.opened() == SIDE * SIDE - MINES { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        for row in 0..SIDE {
            for column in 0..SIDE {
                let (x, y) = BOARD.corner(column, row);
                let mined = self.mines.is_some_and(|mines| mines[row][column]);
                if self.struck.is_some() && mined {
                    let tint = if self.struck == Some((column, row)) {
                        palette::BAD
                    } else {
                        palette::EDGE
                    };
                    frame.rect(x + 1.0, y + 1.0, CELL - 2.0, CELL - 2.0, palette::PANEL);
                    frame.disc(x + CELL * 0.5, y + CELL * 0.5, 9.0, tint);
                } else if self.open[row][column] {
                    frame.bordered(x, y, CELL, CELL, 1.0, palette::BACKDROP, palette::PANEL);
                    let says = self.mines.map_or(0, |mines| count_at(&mines, column, row));
                    if says > 0 {
                        frame.text_center(
                            x + CELL * 0.5,
                            y + CELL * 0.5 - 8.0,
                            &says.to_string(),
                            2,
                            COUNTS[says - 1],
                        );
                    }
                } else {
                    let shut = palette::PANEL.mix(palette::MUTED, 0.16);
                    frame.rect(x + 1.0, y + 1.0, CELL - 2.0, CELL - 2.0, shut);
                    frame.rect(x + 1.0, y + 1.0, CELL - 2.0, 2.0, palette::EDGE);
                    frame.rect(x + 1.0, y + 1.0, 2.0, CELL - 2.0, palette::EDGE);
                }
            }
        }
        let (across, down) = BOARD.span();
        frame.outline(BOARD.left, BOARD.top, across, down, 2.0, palette::MUTED);
    }
}

#[cfg(test)]
#[path = "../tests/games/mines.rs"]
mod tests;
