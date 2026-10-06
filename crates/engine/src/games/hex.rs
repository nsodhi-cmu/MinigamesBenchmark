//! Fill hexagons to hem in a runner before it reaches the edge of the board.

use std::collections::VecDeque;

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "hex",
    title: "Hexagon Home",
    brief: "A red runner stands on a six-sided board of hexagons, some already filled. Click an \
            open hexagon to fill it. After every click the runner steps to a neighboring open \
            hexagon on its shortest way to the green outer ring of the board. Where two steps are as \
            short, it takes the one with more shortest ways on from it, and where those tie too, \
            the first going counterclockwise from straight right. If it reaches the outer ring it \
            escapes and the round is over. Fill hexagons until it has no way left to the outer \
            ring. There are three boards.",
    controls: &["left click an open hexagon to fill it"],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Strict, Tag::Puzzle, Tag::Grid],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Hex::new())
}

pub(crate) const RADIUS: i32 = 5;
pub(crate) const CELLS: usize = 91;
/// Counterclockwise on the screen from straight right, as steps in the two axes.
const STEPS: [(i32, i32); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];
const SIZE: f32 = 24.0;
const RIM: Rgb = Rgb(40, 132, 100);
const MIDDLE: (f32, f32) = (frame::WIDTH as f32 * 0.5, frame::HEIGHT as f32 * 0.5);

/// Each board a row at a time from the top: `#` filled, `R` the runner.
pub(crate) const BOARDS: [[&str; 11]; 3] = [
    [
        ".....#",
        "#.##.#.",
        "##..#...",
        "#...#....",
        "#.####....",
        "...........",
        "...#...#.#",
        "....#R#..",
        "###.#...",
        ".#.....",
        "......",
    ],
    [
        "......",
        "......#",
        "...##...",
        "#.....#..",
        "..#.......",
        "......#R.#.",
        "###.#.....",
        ".......#.",
        "..#.....",
        ".#.#.#.",
        ".#....",
    ],
    [
        "...#..",
        "##.....",
        ".#.#..#.",
        "#......##",
        "#......#..",
        "..#...#..##",
        "..........",
        "...#R..#.",
        "##......",
        ".......",
        "......",
    ],
];

/// Every hexagon's place along the two axes, a row at a time from the top.
pub(crate) const PLACES: [(i32, i32); CELLS] = {
    let mut out = [(0, 0); CELLS];
    let (mut at, mut r) = (0, -RADIUS);
    while r <= RADIUS {
        let mut q = -RADIUS;
        while q <= RADIUS {
            if (q + r).abs() <= RADIUS {
                out[at] = (q, r);
                at += 1;
            }
            q += 1;
        }
        r += 1;
    }
    out
};

/// Each hexagon's neighbors in the order of `STEPS`, where the board has them.
const NEIGHBORS: [[Option<usize>; 6]; CELLS] = {
    let mut out = [[None; 6]; CELLS];
    let mut cell = 0;
    while cell < CELLS {
        let (q, r) = PLACES[cell];
        let mut way = 0;
        while way < 6 {
            let (nq, nr) = (q + STEPS[way].0, r + STEPS[way].1);
            let mut other = 0;
            while other < CELLS {
                if PLACES[other].0 == nq && PLACES[other].1 == nr {
                    out[cell][way] = Some(other);
                }
                other += 1;
            }
            way += 1;
        }
        cell += 1;
    }
    out
};

pub(crate) fn neighbors(cell: usize) -> impl Iterator<Item = usize> {
    NEIGHBORS[cell].into_iter().flatten()
}

pub(crate) fn on_rim(cell: usize) -> bool {
    let (q, r) = PLACES[cell];
    q.abs().max(r.abs()).max((q + r).abs()) == RADIUS
}

/// Where the runner steps from `at`, or nothing if it has no way to the rim.
pub(crate) fn step(filled: u128, at: usize) -> Option<usize> {
    let mut far = [u32::MAX; CELLS];
    let mut ways = [0u64; CELLS];
    let mut queue = VecDeque::new();
    for cell in (0..CELLS).filter(|c| on_rim(*c) && filled >> c & 1 == 0) {
        far[cell] = 0;
        ways[cell] = 1;
        queue.push_back(cell);
    }
    while let Some(cell) = queue.pop_front() {
        for next in neighbors(cell).filter(|n| filled >> n & 1 == 0) {
            if far[next] == u32::MAX {
                far[next] = far[cell] + 1;
                queue.push_back(next);
            }
            if far[next] == far[cell] + 1 {
                ways[next] = ways[next].saturating_add(ways[cell]);
            }
        }
    }
    if far[at] == u32::MAX {
        return None;
    }
    neighbors(at).filter(|n| filled >> n & 1 == 0 && far[*n] != u32::MAX).fold(None, |best, n| {
        match best {
            Some(b) if far[n] > far[b] || (far[n] == far[b] && ways[n] <= ways[b]) => Some(b),
            _ => Some(n),
        }
    })
}

/// A board's filled hexagons and where its runner starts.
pub(crate) fn laid(board: usize) -> (u128, usize) {
    let letters: Vec<char> = BOARDS[board].concat().chars().collect();
    let filled = (0..CELLS).filter(|c| letters[*c] == '#').fold(0u128, |m, c| m | 1 << c);
    (filled, letters.iter().position(|c| *c == 'R').expect("every board has a runner"))
}

pub(crate) fn center(cell: usize) -> (f32, f32) {
    let (q, r) = PLACES[cell];
    let root = 3f32.sqrt();
    (MIDDLE.0 + SIZE * root * (q as f32 + r as f32 * 0.5), MIDDLE.1 + SIZE * 1.5 * r as f32)
}

pub(crate) struct Hex {
    pub(crate) board: usize,
    pub(crate) filled: u128,
    pub(crate) runner: usize,
}

impl Hex {
    pub(crate) fn new() -> Self {
        let (filled, runner) = laid(0);
        Hex { board: 0, filled, runner }
    }
}

impl Game for Hex {
    fn update(&mut self, input: &Input) -> Outcome {
        if !input.button_pressed(Button::Left) {
            return Outcome::Playing;
        }
        let (x, y) = input.cursor();
        let Some(cell) = (0..CELLS).find(|c| {
            let (cx, cy) = center(*c);
            (cx - x).powi(2) + (cy - y).powi(2) <= (SIZE * 0.85).powi(2)
        }) else {
            return Outcome::Playing;
        };
        if cell == self.runner || self.filled >> cell & 1 == 1 {
            return Outcome::Playing;
        }
        self.filled |= 1 << cell;
        match step(self.filled, self.runner) {
            Some(next) => {
                self.runner = next;
                if on_rim(next) { Outcome::Lost } else { Outcome::Playing }
            }
            None if self.board + 1 == BOARDS.len() => Outcome::Won,
            None => {
                self.board += 1;
                (self.filled, self.runner) = laid(self.board);
                Outcome::Playing
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        for cell in 0..CELLS {
            let (x, y) = center(cell);
            let tint = if self.filled >> cell & 1 == 1 {
                palette::INK
            } else if on_rim(cell) {
                RIM
            } else {
                palette::EDGE
            };
            frame.corners(x, y, &[SIZE - 2.0; 6], 0.0, tint);
        }
        let (x, y) = center(self.runner);
        frame.disc(x, y, SIZE * 0.55, palette::BAD);
        frame.disc(x, y, SIZE * 0.22, palette::BACKDROP);
    }
}

#[cfg(test)]
#[path = "../tests/games/hex.rs"]
mod tests;
