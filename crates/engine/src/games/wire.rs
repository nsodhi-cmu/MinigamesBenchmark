//! Drag a dot the length of a wire maze without touching it or letting go.

use crate::frame::Frame;
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "wire",
    title: "Wire Wind",
    brief: "A dot sits in the top left corner of a maze made of wire. Press the left button \
            on the dot and keep holding it: while you hold, the dot follows the pointer, but \
            only so fast. Drag the dot to the ring in the bottom right corner. Touch the \
            wire with it, or let go of the button before it is there, and the round is over.",
    controls: &["hold the left button on the dot and drag it", "the dot follows, but only so fast"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Hostile, Tag::Navigate, Tag::Aim, Tag::Drag],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Wire::new(seed))
}

const COLUMNS: usize = 14;
const ROWS: usize = 10;
const CELL: f32 = 44.0;
const ORIGIN_X: f32 = 12.0;
const ORIGIN_Y: f32 = 20.0;
const DOT: f32 = 7.0;
const WIRE: f32 = 3.0;
const PACE: f32 = 300.0;
/// How near the pointer has to be to the dot to take hold of it.
const GRAB: f32 = 16.0;
const RING: f32 = 13.0;
/// The fewest and the most steps the way through a maze may take.
const SHORTEST: usize = 40;
const LONGEST: usize = 64;

type Grid = [[bool; COLUMNS]; ROWS];
type Wall = ((f32, f32), (f32, f32));

fn center_of(column: usize, row: usize) -> (f32, f32) {
    (ORIGIN_X + (column as f32 + 0.5) * CELL, ORIGIN_Y + (row as f32 + 0.5) * CELL)
}

/// A maze with one route between any two cells, dug as a tree.
fn dig(rng: &mut Rng) -> (Grid, Grid) {
    let (mut right, mut below) = ([[true; COLUMNS]; ROWS], [[true; COLUMNS]; ROWS]);
    let mut seen = [[false; COLUMNS]; ROWS];
    let mut stack = vec![(0usize, 0usize)];
    seen[0][0] = true;
    while let Some(&(column, row)) = stack.last() {
        let ways: Vec<(usize, usize)> =
            beside(column, row).into_iter().filter(|&(nc, nr)| !seen[nr][nc]).collect();
        let Some(&(nc, nr)) = ways.get(rng.below(ways.len().max(1))) else {
            stack.pop();
            continue;
        };
        match (nc.cmp(&column), nr.cmp(&row)) {
            (std::cmp::Ordering::Greater, _) => right[row][column] = false,
            (std::cmp::Ordering::Less, _) => right[row][nc] = false,
            (_, std::cmp::Ordering::Greater) => below[row][column] = false,
            _ => below[nr][column] = false,
        }
        seen[nr][nc] = true;
        stack.push((nc, nr));
    }
    (right, below)
}

/// Whether a cell and the one next to it are joined.
fn joined(right: &Grid, below: &Grid, from: (usize, usize), to: (usize, usize)) -> bool {
    match (to.0 as i32 - from.0 as i32, to.1 as i32 - from.1 as i32) {
        (1, 0) => !right[from.1][from.0],
        (-1, 0) => !right[from.1][to.0],
        (0, 1) => !below[from.1][from.0],
        (0, -1) => !below[to.1][from.0],
        _ => false,
    }
}

fn beside(column: usize, row: usize) -> Vec<(usize, usize)> {
    [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .filter_map(|(dc, dr)| {
            let (nc, nr) = (column as i32 + dc, row as i32 + dr);
            let inside = nc >= 0 && nr >= 0 && (nc as usize) < COLUMNS && (nr as usize) < ROWS;
            inside.then_some((nc as usize, nr as usize))
        })
        .collect()
}

/// How many cells every cell is from the far corner, walking the maze.
fn distances(right: &Grid, below: &Grid) -> [[usize; COLUMNS]; ROWS] {
    let mut away = [[usize::MAX; COLUMNS]; ROWS];
    let mut edge = vec![(COLUMNS - 1, ROWS - 1)];
    away[ROWS - 1][COLUMNS - 1] = 0;
    let mut step = 0;
    while !edge.is_empty() {
        step += 1;
        let mut next = Vec::new();
        for (column, row) in edge {
            for (nc, nr) in beside(column, row) {
                if away[nr][nc] == usize::MAX && joined(right, below, (column, row), (nc, nr)) {
                    away[nr][nc] = step;
                    next.push((nc, nr));
                }
            }
        }
        edge = next;
    }
    away
}

fn wires(right: &Grid, below: &Grid) -> Vec<Wall> {
    let corner =
        |column: usize, row: usize| (ORIGIN_X + column as f32 * CELL, ORIGIN_Y + row as f32 * CELL);
    let mut out = vec![
        (corner(0, 0), corner(COLUMNS, 0)),
        (corner(0, ROWS), corner(COLUMNS, ROWS)),
        (corner(0, 0), corner(0, ROWS)),
        (corner(COLUMNS, 0), corner(COLUMNS, ROWS)),
    ];
    for (row, line) in right.iter().enumerate() {
        for (column, walled) in line.iter().enumerate().take(COLUMNS - 1) {
            if *walled {
                out.push((corner(column + 1, row), corner(column + 1, row + 1)));
            }
        }
    }
    for (row, line) in below.iter().enumerate().take(ROWS - 1) {
        for (column, walled) in line.iter().enumerate() {
            if *walled {
                out.push((corner(column, row + 1), corner(column + 1, row + 1)));
            }
        }
    }
    out
}

/// How far a spot is from the nearest point of a length of wire.
fn near(at: (f32, f32), (a, b): Wall) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let along = dx * dx + dy * dy;
    let part = if along == 0.0 {
        0.0
    } else {
        (((at.0 - a.0) * dx + (at.1 - a.1) * dy) / along).clamp(0.0, 1.0)
    };
    libm::hypotf(at.0 - (a.0 + dx * part), at.1 - (a.1 + dy * part))
}

struct Wire {
    wires: Vec<Wall>,
    at: (f32, f32),
    goal: (f32, f32),
    pointer: (f32, f32),
    carrying: bool,
}

/// A maze whose way through is neither a dash nor a slog.
fn lay(seed: u64) -> (Grid, Grid) {
    let mut rng = Rng::new(seed);
    loop {
        let (right, below) = dig(&mut rng);
        if (SHORTEST..=LONGEST).contains(&distances(&right, &below)[0][0]) {
            return (right, below);
        }
    }
}

impl Wire {
    fn new(seed: u64) -> Self {
        let (right, below) = lay(seed);
        Wire {
            wires: wires(&right, &below),
            at: center_of(0, 0),
            goal: center_of(COLUMNS - 1, ROWS - 1),
            pointer: center_of(0, 0),
            carrying: false,
        }
    }
}

impl Game for Wire {
    fn update(&mut self, input: &Input) -> Outcome {
        let holding = input.button_held(Button::Left);
        let pointer = input.cursor();
        self.pointer = pointer;
        if !self.carrying {
            let grabbed = input.button_pressed(Button::Left)
                && holding
                && libm::hypotf(pointer.0 - self.at.0, pointer.1 - self.at.1) <= GRAB;
            self.carrying = grabbed;
            return Outcome::Playing;
        }
        if !holding {
            return Outcome::Lost;
        }

        self.at = walk::toward(self.at, pointer, PACE * TICK);
        if self.wires.iter().any(|wire| near(self.at, *wire) < DOT + WIRE * 0.5) {
            return Outcome::Lost;
        }
        if libm::hypotf(self.at.0 - self.goal.0, self.at.1 - self.goal.1) < RING - DOT {
            return Outcome::Won;
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        frame.ring(self.goal.0, self.goal.1, RING, 3.0, palette::GOOD);
        for (a, b) in &self.wires {
            frame.line(a.0, a.1, b.0, b.1, WIRE, palette::MUTED);
        }
        if self.carrying {
            // The line the dot is being pulled along, so that how far the
            // pointer has run ahead of it can be seen.
            frame.line(self.at.0, self.at.1, self.pointer.0, self.pointer.1, 1.0, palette::EDGE);
        } else {
            frame.ring(self.at.0, self.at.1, DOT + 6.0, 2.0, palette::MARK);
        }
        frame.disc(self.at.0, self.at.1, DOT, palette::PLAYER);
    }
}

#[cfg(test)]
#[path = "../tests/games/wire.rs"]
mod tests;
