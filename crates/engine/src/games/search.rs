//! Find five words hidden in a grid of letters.

use crate::board::Board;
use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;
use crate::words::WORDS;

pub const SPEC: Spec = Spec {
    name: "search",
    title: "Word Search",
    brief: "Five words are hidden in a ten by ten grid of letters, and listed beside it. A \
            word runs in a straight line in any of eight directions: across, down, or \
            diagonally, forward or backward. Press on the first letter of a word and drag \
            to its last letter to find it. Find all five.",
    controls: &["press on a word's first letter and drag to its last"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Language, Tag::Grid, Tag::Drag],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Search::new(seed))
}

pub(crate) const SIDE: usize = 10;
pub(crate) const HIDDEN: usize = 5;
const LENGTH: usize = 5;
pub(crate) const CELL: f32 = 42.0;
pub(crate) const BOARD: Board =
    Board::at(30.0, (frame::HEIGHT as f32 - CELL * SIDE as f32) * 0.5, SIDE, SIDE, CELL);
const LIST_LEFT: f32 = 480.0;
/// The list of words is spaced this far apart and centered like the grid.
const LIST_PITCH: f32 = 56.0;

pub(crate) const WAYS: [(i32, i32); 8] =
    [(0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1)];

const TRACES: [Rgb; HIDDEN] =
    [Rgb(40, 90, 150), Rgb(40, 120, 80), Rgb(140, 100, 30), Rgb(120, 60, 140), Rgb(140, 60, 80)];

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) struct Hidden {
    pub(crate) word: &'static str,
    pub(crate) first: usize,
    pub(crate) way: (i32, i32),
}

/// The squares `LENGTH` long from `first` going `way`, if they all lie on the grid.
pub(crate) fn line(first: usize, way: (i32, i32)) -> Option<Vec<usize>> {
    let (row, column) = ((first / SIDE) as i32, (first % SIDE) as i32);
    (0..LENGTH as i32)
        .map(|step| {
            let (r, c) = (row + way.0 * step, column + way.1 * step);
            BOARD.holds(c, r).then(|| r as usize * SIDE + c as usize)
        })
        .collect()
}

/// A grid with five words hidden in it, each of which can be read on it in
/// exactly one place and one direction.
pub(crate) fn deal(rng: &mut Rng) -> ([u8; SIDE * SIDE], Vec<Hidden>) {
    'again: loop {
        let mut grid = [0u8; SIDE * SIDE];
        for letter in grid.iter_mut() {
            *letter = b'a' + rng.below(26) as u8;
        }
        let mut fixed = [false; SIDE * SIDE];
        let mut hidden: Vec<Hidden> = Vec::new();
        for word in rng.take(WORDS, HIDDEN) {
            let placed = (0..200).find_map(|_| {
                let (first, way) = (rng.below(SIDE * SIDE), WAYS[rng.below(WAYS.len())]);
                let cells = line(first, way)?;
                let fits = cells
                    .iter()
                    .zip(word.bytes())
                    .all(|(cell, letter)| !fixed[*cell] || grid[*cell] == letter);
                fits.then_some((first, way, cells))
            });
            let Some((first, way, cells)) = placed else { continue 'again };
            for (cell, letter) in cells.iter().zip(word.bytes()) {
                grid[*cell] = letter;
                fixed[*cell] = true;
            }
            hidden.push(Hidden { word, first, way });
        }
        if hidden.iter().all(|h| sightings(&grid, h.word) == 1) {
            return (grid, hidden);
        }
    }
}

/// How many places and directions `word` can be read in on the grid.
pub(crate) fn sightings(grid: &[u8; SIDE * SIDE], word: &str) -> usize {
    (0..SIDE * SIDE)
        .flat_map(|first| WAYS.iter().filter_map(move |way| line(first, *way)))
        .filter(|cells| cells.iter().zip(word.bytes()).all(|(cell, letter)| grid[*cell] == letter))
        .count()
}

pub(crate) fn middle(cell: usize) -> (f32, f32) {
    BOARD.middle(cell % SIDE, cell / SIDE)
}

fn under(at: (f32, f32)) -> Option<usize> {
    BOARD.under(at).map(|(column, row)| row * SIDE + column)
}

pub(crate) struct Search {
    grid: [u8; SIDE * SIDE],
    pub(crate) hidden: Vec<Hidden>,
    pub(crate) found: [bool; HIDDEN],
    dragging: Option<usize>,
    pointer: (f32, f32),
}

impl Search {
    pub(crate) fn new(seed: u64) -> Self {
        let (grid, hidden) = deal(&mut Rng::new(seed));
        Search { grid, hidden, found: [false; HIDDEN], dragging: None, pointer: (0.0, 0.0) }
    }
}

fn trace(frame: &mut Frame, from: (f32, f32), to: (f32, f32), tint: Rgb) {
    let weight = CELL * 0.7;
    frame.line(from.0, from.1, to.0, to.1, weight, tint);
    frame.disc(from.0, from.1, weight * 0.5, tint);
    frame.disc(to.0, to.1, weight * 0.5, tint);
}

impl Game for Search {
    fn update(&mut self, input: &Input) -> Outcome {
        self.pointer = input.cursor();
        if input.button_pressed(Button::Left) {
            self.dragging = under(self.pointer);
        }
        if !input.button_held(Button::Left)
            && let Some(first) = self.dragging.take()
            && let Some(last) = under(self.pointer)
        {
            for (which, hidden) in self.hidden.iter().enumerate() {
                if hidden.first == first && line(first, hidden.way).unwrap()[LENGTH - 1] == last {
                    self.found[which] = true;
                }
            }
        }
        if self.found.iter().all(|found| *found) { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let (across, down) = BOARD.span();
        frame.rect(BOARD.left - 6.0, BOARD.top - 6.0, across + 12.0, down + 12.0, palette::PANEL);
        for (which, hidden) in self.hidden.iter().enumerate() {
            if self.found[which] {
                let cells = line(hidden.first, hidden.way).unwrap();
                trace(frame, middle(cells[0]), middle(cells[LENGTH - 1]), TRACES[which]);
            }
        }
        if let Some(first) = self.dragging {
            trace(frame, middle(first), self.pointer, palette::EDGE);
        }
        for (cell, letter) in self.grid.iter().enumerate() {
            let (x, y) = middle(cell);
            let text = (*letter as char).to_ascii_uppercase().to_string();
            frame.text_center(x, y - frame::text_height(2) * 0.5, &text, 2, palette::INK);
        }

        for (which, hidden) in self.hidden.iter().enumerate() {
            let tall = (HIDDEN - 1) as f32 * LIST_PITCH + frame::text_height(3);
            let y = (frame::HEIGHT as f32 - tall) * 0.5 + which as f32 * LIST_PITCH;
            let text = hidden.word.to_ascii_uppercase();
            let tint = if self.found[which] { palette::MUTED } else { palette::INK };
            frame.text(LIST_LEFT, y, &text, 3, tint);
            if self.found[which] {
                let strike = y + frame::text_height(3) * 0.5;
                frame.rect(
                    LIST_LEFT - 6.0,
                    strike - 2.0,
                    frame::text_width(&text, 3) + 12.0,
                    4.0,
                    palette::GOOD,
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/search.rs"]
mod tests;
