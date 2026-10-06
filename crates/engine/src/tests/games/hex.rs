use std::collections::HashMap;

use super::{CELLS, Hex, center, neighbors, on_rim, step};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

/// Clicks enough to win each board, which the search is held to.
const CLICKS: [u32; 3] = [5, 7, 9];

/// Searches for a win within a number of clicks, remembering positions already
/// shown to be lost within as many.
struct Search {
    lost: HashMap<(u128, usize), u32>,
}

impl Search {
    /// The hexagons worth clicking: none when the runner has two ways onto the
    /// rim at once, since only one can be filled; only that one when it has one.
    fn choices(filled: u128, runner: usize) -> Vec<usize> {
        let rim: Vec<usize> =
            neighbors(runner).filter(|n| on_rim(*n) && filled >> n & 1 == 0).collect();
        if rim.len() >= 2 {
            return Vec::new();
        }
        if rim.len() == 1 {
            return rim;
        }
        let mut reach = vec![runner];
        let mut seen = 1u128 << runner;
        while let Some(cell) = reach.pop() {
            for next in neighbors(cell) {
                if filled >> next & 1 == 0 && seen >> next & 1 == 0 {
                    seen |= 1 << next;
                    reach.push(next);
                }
            }
        }
        (0..CELLS).filter(|c| *c != runner && seen >> c & 1 == 1).collect()
    }

    fn click_wins(&mut self, filled: u128, runner: usize, cell: usize, clicks: u32) -> bool {
        let filled = filled | 1 << cell;
        match step(filled, runner) {
            None => true,
            Some(next) if on_rim(next) => false,
            Some(next) => self.wins(filled, next, clicks - 1),
        }
    }

    fn wins(&mut self, filled: u128, runner: usize, clicks: u32) -> bool {
        if clicks == 0 || self.lost.get(&(filled, runner)).is_some_and(|c| *c >= clicks) {
            return false;
        }
        for cell in Search::choices(filled, runner) {
            if self.click_wins(filled, runner, cell, clicks) {
                return true;
            }
        }
        self.lost.insert((filled, runner), clicks);
        false
    }
}

/// Clicking a win the search finds on each board clears all three.
#[test]
fn clicking_a_win_on_each_board_clears_all_three() {
    let mut game = Hex::new();
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    for (board, clicks) in CLICKS.iter().enumerate() {
        assert_eq!(game.board, board);
        let mut search = Search { lost: HashMap::new() };
        for left in (1..=*clicks).rev() {
            let (filled, runner) = (game.filled, game.runner);
            let cell = Search::choices(filled, runner)
                .into_iter()
                .find(|cell| search.click_wins(filled, runner, *cell, left))
                .expect("a win carries on");
            let (x, y) = center(cell);
            input.apply(Action::Click { button: Button::Left, x, y });
            outcome = game.update(&input);
            input.settle();
        }
    }
    assert_eq!(outcome, Outcome::Won);
}
