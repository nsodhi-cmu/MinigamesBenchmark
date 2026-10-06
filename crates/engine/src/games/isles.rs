//! Slide across an isle of ice, stopping only where a rock is in the way.

use crate::board::Board;
use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::map::Map;
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "isles",
    title: "Ice Isles",
    brief: "A square of ice floats on dark water with rocks frozen into it. You are the \
            orange square and the green square is where you are going. Press an arrow and \
            you slide that way at a steady pace until a rock stops you. Nothing at the edge \
            of the ice stops you: a slide that reaches it goes into the water and the round \
            is over. Reach the green square on three isles.",
    controls: &["an arrow slides you that way until a rock stops you"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[
        Tag::Bounded,
        Tag::Strict,
        Tag::Puzzle,
        Tag::Navigate,
        Tag::Overhead,
        Tag::Grid,
        Tag::Slippery,
    ],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Isles::new())
}

pub(crate) const SIDE: usize = 8;
const CELL: f32 = 48.0;
const BOARD: Board = Board::centered(SIDE, SIDE, CELL);
/// Cells slid in a second.
const PACE: f32 = 10.0;

/// `#` a rock, `S` where the slide starts, `E` the square to reach.
pub(crate) const BOARDS: [[&str; SIDE]; 3] = [
    [
        "..#..#..", "...#....", "#.......", "......#.", "..#.....", "....E...", "....#.S.",
        ".#......",
    ],
    [
        ".#....#.", "...##...", "#.......", "....#E..", ".#......", ".......#", ".S..#...",
        "........",
    ],
    [
        "....#...", "......#.", ".E......", "........", "...S.#..", ".......#", "......#.",
        "..##....",
    ],
];

const ICE: Rgb = Rgb(178, 224, 244);
const SHINE: Rgb = Rgb(232, 248, 255);
const WATER: Rgb = Rgb(22, 48, 92);
const ROCK: Rgb = Rgb(112, 118, 130);

fn map(board: usize) -> Map<'static> {
    Map::new(&BOARDS[board], SIDE, CELL)
}

/// The cell carrying `mark`, across then down.
pub(crate) fn find(board: usize, mark: u8) -> (i32, i32) {
    let map = map(board);
    map.board.cell_at(map.find(mark))
}

/// What one cell farther along a slide would meet.
pub(crate) enum Ahead {
    Ice,
    Rock,
    Goal,
    Water,
}

pub(crate) fn ahead(board: usize, (column, row): (i32, i32)) -> Ahead {
    match map(board).mark(column, row) {
        None => Ahead::Water,
        Some(b'#') => Ahead::Rock,
        Some(b'E') => Ahead::Goal,
        Some(_) => Ahead::Ice,
    }
}

/// A square set in from the edges of the cell it is centered on.
fn square(frame: &mut Frame, (cx, cy): (f32, f32), inset: f32, weight: f32, fill: Rgb, edge: Rgb) {
    let half = CELL * 0.5 - inset;
    frame.bordered(cx - half, cy - half, half * 2.0, half * 2.0, weight, fill, edge);
}

pub(crate) struct Isles {
    pub(crate) done: usize,
    pub(crate) at: (i32, i32),
    /// The way the slide is going and how far along the next cell it is.
    pub(crate) slide: Option<((i32, i32), f32)>,
    settled: Option<Outcome>,
}

impl Isles {
    pub(crate) fn new() -> Self {
        Isles { done: 0, at: find(0, b'S'), slide: None, settled: None }
    }

    fn board(&self) -> usize {
        self.done.min(BOARDS.len() - 1)
    }

    fn arrive(&mut self) -> Outcome {
        self.done += 1;
        self.slide = None;
        if self.done == BOARDS.len() {
            return Outcome::Won;
        }
        self.at = find(self.done, b'S');
        Outcome::Playing
    }
}

impl Game for Isles {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some(settled) = self.settled {
            return settled;
        }
        let Some((step, along)) = self.slide else {
            if let Some((_, step)) = input.arrow()
                && !matches!(
                    ahead(self.board(), (self.at.0 + step.0, self.at.1 + step.1)),
                    Ahead::Rock
                )
            {
                self.slide = Some((step, 0.0));
            }
            return Outcome::Playing;
        };
        let mut along = along + PACE * TICK;
        while along >= 1.0 {
            along -= 1.0;
            self.at = (self.at.0 + step.0, self.at.1 + step.1);
            let outcome = match ahead(self.board(), self.at) {
                Ahead::Water => Outcome::Lost,
                Ahead::Goal => self.arrive(),
                _ => Outcome::Playing,
            };
            if outcome != Outcome::Playing {
                self.settled = Some(outcome);
            }
            if outcome != Outcome::Playing || self.slide.is_none() {
                return outcome;
            }
            if matches!(ahead(self.board(), (self.at.0 + step.0, self.at.1 + step.1)), Ahead::Rock)
            {
                self.slide = None;
                return Outcome::Playing;
            }
        }
        self.slide = Some((step, along));
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        frame.rect(0.0, 0.0, 640.0, 480.0, WATER);
        let (left, top) = (BOARD.left, BOARD.top);
        let (wide, tall) = BOARD.span();
        frame.rect(left - 4.0, top - 4.0, wide + 8.0, tall + 8.0, ICE.mix(WATER, 0.5));
        frame.rect(left, top, wide, tall, ICE);
        for row in 0..SIDE {
            for column in 0..SIDE {
                let (x, y) = BOARD.corner(column, row);
                frame.rect(x, y, CELL, 1.0, SHINE);
                frame.rect(x, y, 1.0, CELL, SHINE);
                let streak = match column + row {
                    5 => 5.0,
                    6 | 11 => 2.0,
                    _ => continue,
                };
                frame.line(x + CELL, y, x, y + CELL, streak, ICE.mix(SHINE, 0.7));
            }
        }
        let board = self.board();
        for (row, line) in BOARDS[board].iter().enumerate() {
            for (column, letter) in line.chars().enumerate() {
                let (cx, cy) = BOARD.middle(column, row);
                match letter {
                    '#' => {
                        frame.disc(cx, cy + 3.0, 17.0, ROCK.mix(WATER, 0.5));
                        frame.disc(cx, cy, 16.0, ROCK);
                        frame.disc(cx - 5.0, cy - 5.0, 6.0, ROCK.mix(SHINE, 0.4));
                    }
                    'E' => square(frame, (cx, cy), 6.0, 3.0, palette::GOOD, palette::INK),
                    _ => {}
                }
            }
        }
        let (step, along) = self.slide.unwrap_or(((0, 0), 0.0));
        let x = left + (self.at.0 as f32 + step.0 as f32 * along + 0.5) * CELL;
        let y = top + (self.at.1 as f32 + step.1 as f32 * along + 0.5) * CELL;
        square(frame, (x, y), 8.0, 2.0, palette::ORANGE, palette::BACKDROP);
        frame.pips(320.0, 464.0, BOARDS.len() as u32, self.done as u32, palette::GOOD);
    }
}

#[cfg(test)]
#[path = "../tests/games/isles.rs"]
mod tests;
