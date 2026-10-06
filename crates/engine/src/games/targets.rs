//! Fifteen marks on a board, seen through a scope.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "targets",
    title: "Target Practice",
    brief: "Fifteen marks are spread over a board. The scope shows only what is around the \
            crosshair, so sweep it about to find them, and put the crosshair on a mark and \
            click to take it down. A few of them wander. Missing costs you nothing but the \
            time it took.",
    controls: &["move the pointer to aim", "left click to fire"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Aim, Tag::Explore],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Targets::new(seed))
}

const MARKS: usize = 15;
const MARK: f32 = 13.0;
const SCOPE: f32 = 100.0;
/// The board the marks stand on: left, top, right, bottom.
const BOARD: (f32, f32, f32, f32) =
    (24.0, 16.0, frame::WIDTH as f32 - 24.0, frame::HEIGHT as f32 - 16.0);
/// How far a mark stands in from the edge of the board.
const MARGIN: f32 = MARK + 10.0;
/// The coarse grid a mark is placed on, one to a cell, and how far about its
/// cell it may be shifted. The shift is small enough against the spacing that
/// two marks always start more than four times a mark's radius apart.
const ACROSS: usize = 6;
const DOWN: usize = 5;
const JITTER: (f32, f32) = (19.0, 11.0);

struct Mark {
    at: (f32, f32),
    drift: f32,
}

struct Targets {
    marks: Vec<Mark>,
    eye: (f32, f32),
    splash: Option<((f32, f32), u32, bool)>,
}

impl Targets {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let span = (
            (BOARD.2 - BOARD.0 - MARGIN * 2.0) / ACROSS as f32,
            (BOARD.3 - BOARD.1 - MARGIN * 2.0) / DOWN as f32,
        );
        let cells: Vec<usize> = (0..ACROSS * DOWN).collect();
        let marks: Vec<Mark> = rng
            .take(&cells, MARKS)
            .into_iter()
            .enumerate()
            .map(|(which, cell)| {
                let (column, row) = (cell % ACROSS, cell / ACROSS);
                let at = (
                    BOARD.0
                        + MARGIN
                        + (column as f32 + 0.5) * span.0
                        + rng.range(-JITTER.0, JITTER.0),
                    BOARD.1 + MARGIN + (row as f32 + 0.5) * span.1 + rng.range(-JITTER.1, JITTER.1),
                );
                let drift = if which.is_multiple_of(4) {
                    (if rng.chance(0.5) { -1.0 } else { 1.0 }) * rng.range(26.0, 54.0)
                } else {
                    0.0
                };
                Mark { at, drift }
            })
            .collect();
        Targets { marks, eye: (frame::WIDTH as f32 * 0.5, 240.0), splash: None }
    }
}

impl Game for Targets {
    fn update(&mut self, input: &Input) -> Outcome {
        for mark in &mut self.marks {
            mark.at.0 += mark.drift * TICK;
            let (low, high) = (BOARD.0 + MARGIN, BOARD.2 - MARGIN);
            walk::bounced(&mut mark.at.0, &mut mark.drift, low, high, 0.0, 1.0);
        }
        if let Some((at, left, hit)) = self.splash {
            self.splash = (left > 1).then_some((at, left - 1, hit));
        }

        self.eye = input.cursor();
        if input.button_pressed(Button::Left) {
            let (x, y) = input.cursor();
            // Whichever is nearest the click, so a mark behind another one
            // cannot be taken for it.
            let struck = self
                .marks
                .iter()
                .enumerate()
                .map(|(slot, mark)| (slot, libm::hypotf(mark.at.0 - x, mark.at.1 - y)))
                .filter(|(_, away)| *away <= MARK)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(slot, _)| slot);
            self.splash = Some(((x, y), 21, struck.is_some()));
            if let Some(slot) = struck {
                self.marks.swap_remove(slot);
                if self.marks.is_empty() {
                    return Outcome::Won;
                }
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        let (left, top, right, bottom) = BOARD;
        frame.bordered(left, top, right - left, bottom - top, 3.0, palette::PANEL, palette::MUTED);

        for mark in &self.marks {
            frame.disc(mark.at.0, mark.at.1, MARK, palette::INK);
            frame.disc(mark.at.0, mark.at.1, MARK * 0.62, palette::BAD);
            frame.disc(mark.at.0, mark.at.1, MARK * 0.24, palette::INK);
        }
        if let Some((at, left, hit)) = self.splash {
            let tint = if hit { palette::GOOD } else { palette::MUTED };
            frame.ring(at.0, at.1, 20.0 - left as f32 * TICK * 40.0, 3.0, tint);
        }

        // Everything outside the scope is shut out, a row at a time.
        let (cx, cy) = self.eye;
        for row in 0..frame::HEIGHT {
            let dy = row as f32 + 0.5 - cy;
            if dy.abs() >= SCOPE {
                frame.rect(0.0, row as f32, width, 1.0, palette::BACKDROP);
                continue;
            }
            let reach = (SCOPE * SCOPE - dy * dy).sqrt();
            frame.rect(0.0, row as f32, cx - reach, 1.0, palette::BACKDROP);
            frame.rect(cx + reach, row as f32, width - cx - reach, 1.0, palette::BACKDROP);
        }
        frame.ring(cx, cy, SCOPE, 4.0, palette::EDGE);
        frame.line(cx - 26.0, cy, cx - 8.0, cy, 2.0, palette::INK);
        frame.line(cx + 8.0, cy, cx + 26.0, cy, 2.0, palette::INK);
        frame.line(cx, cy - 26.0, cx, cy - 8.0, 2.0, palette::INK);
        frame.line(cx, cy + 8.0, cx, cy + 26.0, 2.0, palette::INK);
    }
}

#[cfg(test)]
#[path = "../tests/games/targets.rs"]
mod tests;
