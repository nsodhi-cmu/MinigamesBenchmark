use super::{BLANKS, CELLS, Grid, Latin, SIDE, correct, deal};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input, Key};
use crate::rng::Rng;

const NUMBERS: [Key; SIDE] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6];

/// Every way of filling the grid in, counted by trying each number in the
/// first empty square, up to a couple so that more than one is noticed.
fn ways(grid: &mut Grid) -> usize {
    let Some(cell) = (0..CELLS).find(|cell| grid[*cell] == 0) else {
        return usize::from(correct(grid));
    };
    let (row, column) = (cell / SIDE, cell % SIDE);
    let mut found = 0;
    for number in 1..=SIDE as u8 {
        let clash = (0..SIDE)
            .any(|at| grid[row * SIDE + at] == number || grid[at * SIDE + column] == number);
        if clash {
            continue;
        }
        grid[cell] = number;
        found += ways(grid);
        grid[cell] = 0;
        if found > 1 {
            break;
        }
    }
    found
}

/// A dealt grid gives away enough that the rest follows without guessing, and
/// no more: the numbers left out have one way to be put back.
#[test]
fn every_grid_dealt_can_be_reasoned_out_and_has_one_answer() {
    for seed in 0..40u64 {
        let (answer, given) = deal(&mut Rng::new(seed));
        assert!(correct(&answer), "seed {seed} was dealt from a grid that repeats a number");
        assert_eq!(ways(&mut given.clone()), 1, "seed {seed} can be filled in more than one way");
        let blank = given.iter().filter(|number| **number == 0).count();
        assert_eq!(blank, BLANKS, "seed {seed} was left {blank} squares of {CELLS} to fill");
    }
}

/// Clicking each blank square and pressing its number fills the grid and wins.
#[test]
fn writing_the_answer_in_fills_the_grid_and_wins() {
    for seed in 0..8u64 {
        let (answer, given) = deal(&mut Rng::new(seed));
        let mut game = Latin::new(seed);
        let mut input = Input::new();
        let mut outcome = Outcome::Playing;
        for cell in (0..CELLS).filter(|cell| given[*cell] == 0) {
            assert_eq!(outcome, Outcome::Playing);
            let (x, y) = super::BOARD.middle(cell % SIDE, cell / SIDE);
            input.apply(Action::Click { button: Button::Left, x, y });
            game.update(&input);
            input.settle();
            input.apply(Action::Tap(NUMBERS[answer[cell] as usize - 1]));
            outcome = game.update(&input);
            input.settle();
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed}");
    }
}

#[test]
fn a_number_that_repeats_one_in_its_row_leaves_the_grid_unfinished() {
    let seed = 3;
    let (answer, given) = deal(&mut Rng::new(seed));
    let mut game = Latin::new(seed);
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    let blanks: Vec<usize> = (0..CELLS).filter(|cell| given[*cell] == 0).collect();
    let wrong = *blanks.first().expect("a grid has squares to fill");
    for cell in blanks {
        let (x, y) = super::BOARD.middle(cell % SIDE, cell / SIDE);
        input.apply(Action::Click { button: Button::Left, x, y });
        game.update(&input);
        input.settle();
        let number = if cell == wrong { answer[cell] % SIDE as u8 + 1 } else { answer[cell] };
        input.apply(Action::Tap(NUMBERS[number as usize - 1]));
        outcome = game.update(&input);
        input.settle();
    }
    assert_eq!(outcome, Outcome::Playing, "a grid that repeats a number was taken as done");

    let (x, y) = super::BOARD.middle(wrong % SIDE, wrong / SIDE);
    input.apply(Action::Click { button: Button::Left, x, y });
    game.update(&input);
    input.settle();
    input.apply(Action::Tap(Key::Backspace));
    game.update(&input);
    input.settle();
    input.apply(Action::Tap(NUMBERS[answer[wrong] as usize - 1]));
    assert_eq!(
        game.update(&input),
        Outcome::Won,
        "rubbing out and writing again did not finish it"
    );
}
