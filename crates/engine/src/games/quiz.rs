//! Find the two numbers on the board that make the total.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "quiz",
    title: "Quantitative Quiz",
    brief: "A board of nine numbers sits under a total. Exactly one pair of them adds \
            up to that total. Click one, then the other. Five totals finish the round; \
            one wrong pair ends it.",
    controls: &["left click a number to pick it", "click it again to put it back"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Strict, Tag::Quiz],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Ledger::new(seed))
}

const SIDE: usize = 3;
const CELLS: usize = SIDE * SIDE;
const ROUNDS: u32 = 5;
const BOX: f32 = 124.0;
const GAP: f32 = 14.0;
const TOP: f32 = 162.0;

struct Ledger {
    rng: Rng,
    board: [u32; CELLS],
    total: u32,
    picked: Option<usize>,
    solved: u32,
}

/// A board whose answer is the only pair adding to the total. Every pair that
/// would serve is gathered and one is drawn, so the answer is as likely to sit
/// in one corner of the board as in another.
pub(crate) fn lay(rng: &mut Rng) -> ([u32; CELLS], u32, (usize, usize)) {
    let two_figure: Vec<u32> = (10..100).collect();
    let board: [u32; CELLS] =
        rng.take(&two_figure, CELLS).try_into().expect("as many are dealt as the board holds");
    let pairs = || (0..CELLS).flat_map(|a| (a + 1..CELLS).map(move |b| (a, b)));
    // The two smallest numbers on the board add to less than any other pair
    // can, so a board of numbers that all differ always has an answer.
    let answers: Vec<(usize, usize)> = pairs()
        .filter(|(a, b)| {
            let total = board[*a] + board[*b];
            pairs().filter(|(i, j)| board[*i] + board[*j] == total).count() == 1
        })
        .collect();
    let (a, b) = answers[rng.below(answers.len())];
    (board, board[a] + board[b], (a, b))
}

impl Ledger {
    fn new(seed: u64) -> Self {
        let mut ledger =
            Ledger { rng: Rng::new(seed), board: [0; CELLS], total: 0, picked: None, solved: 0 };
        ledger.deal();
        ledger
    }

    fn deal(&mut self) {
        let (board, total, _) = lay(&mut self.rng);
        self.board = board;
        self.total = total;
        self.picked = None;
    }

    fn origin() -> (f32, f32) {
        let span = SIDE as f32 * BOX + (SIDE as f32 - 1.0) * GAP;
        (frame::WIDTH as f32 * 0.5 - span * 0.5, TOP)
    }

    fn cell_at(x: f32, y: f32) -> Option<usize> {
        (0..CELLS).find(|slot| {
            let (cx, cy) = Self::cell_box(*slot);
            x >= cx && y >= cy && x <= cx + BOX && y <= cy + BOX * 0.7
        })
    }

    fn cell_box(slot: usize) -> (f32, f32) {
        let (ox, oy) = Self::origin();
        (ox + (slot % SIDE) as f32 * (BOX + GAP), oy + (slot / SIDE) as f32 * (BOX * 0.7 + GAP))
    }
}

impl Game for Ledger {
    fn update(&mut self, input: &Input) -> Outcome {
        if !input.button_pressed(Button::Left) {
            return Outcome::Playing;
        }
        let (x, y) = input.cursor();
        let Some(slot) = Self::cell_at(x, y) else {
            return Outcome::Playing;
        };
        match self.picked {
            None => self.picked = Some(slot),
            Some(first) if first == slot => self.picked = None,
            Some(first) => {
                if self.board[first] + self.board[slot] == self.total {
                    self.solved += 1;
                    if self.solved == ROUNDS {
                        return Outcome::Won;
                    }
                    self.deal();
                } else {
                    return Outcome::Lost;
                }
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let middle = frame::WIDTH as f32 * 0.5;
        frame.text_center(middle, 74.0, "MAKE THIS TOTAL", 1, palette::MUTED);
        frame.text_center(middle, 94.0, &self.total.to_string(), 5, palette::WARN);

        for slot in 0..CELLS {
            let (x, y) = Self::cell_box(slot);
            let chosen = self.picked == Some(slot);
            let fill = if chosen { palette::MARK } else { palette::PANEL };
            frame.bordered(x, y, BOX, BOX * 0.7, 2.0, fill, palette::EDGE);
            frame.text_center(
                x + BOX * 0.5,
                y + BOX * 0.35 - 8.0,
                &self.board[slot].to_string(),
                2,
                palette::INK,
            );
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/quiz.rs"]
mod tests;
