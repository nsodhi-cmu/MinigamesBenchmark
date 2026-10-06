use super::{Mines, SIDE, reasons_out};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};
use crate::rng::Rng;

/// The ring of squares around the first click is kept clear so that the click
/// opens a region rather than a bare number. That is what the clearing is for,
/// and the only thing about it a player would notice.
#[test]
fn the_first_click_opens_a_region_rather_than_one_square() {
    for seed in 0..40u64 {
        let mut game = Mines::new(seed);
        let first = (4, 4);
        assert_eq!(
            click(&mut game, first),
            Outcome::Playing,
            "seed {seed} lost on the first click"
        );
        let opened = game.open.iter().flatten().filter(|seen| **seen).count();
        assert!(opened > 9, "seed {seed} opened {opened} squares, so the ring was not kept clear");
    }
}

fn click(game: &mut Mines, (column, row): (usize, usize)) -> Outcome {
    let mut input = Input::new();
    let (x, y) = super::BOARD.middle(column, row);
    input.apply(Action::Move { x, y });
    let here = input.cursor();
    input.apply(Action::Press { button: Button::Left, x: here.0, y: here.1 });
    game.update(&input)
}

/// Opening every square that is not a mine finishes the round, and opening one
/// that is ends it.
#[test]
fn opening_the_ninety_safe_squares_finishes_and_a_mine_ends_it() {
    for seed in 0..12u64 {
        let mut game = Mines::new(seed);
        let mut outcome = click(&mut game, (4, 4));
        let mines = game.mines.expect("the mines are laid by the first click");
        for (row, line) in mines.iter().enumerate() {
            for (column, mined) in line.iter().enumerate() {
                if *mined || game.open[row][column] {
                    continue;
                }
                outcome = click(&mut game, (column, row));
                assert_ne!(outcome, Outcome::Lost, "seed {seed} lost on a safe square");
            }
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} opened everything and did not finish");

        let mut struck = Mines::new(seed);
        click(&mut struck, (4, 4));
        let mined = (0..SIDE)
            .flat_map(|row| (0..SIDE).map(move |column| (column, row)))
            .find(|(c, r)| mines[*r][*c])
            .expect("ten mines are laid");
        assert_eq!(click(&mut struck, mined), Outcome::Lost, "seed {seed} forgave a mine");
    }
}

/// Wherever the first click lands, a field that needs no guessing is found in
/// a handful of deals, so laying the mines after the click can never hang.
#[test]
fn a_fair_field_is_found_in_a_handful_of_deals_wherever_the_first_click_lands() {
    for seed in 0..60u64 {
        for first in [(0usize, 0usize), (4, 4), (9, 0), (9, 9), (0, 5), (7, 2)] {
            let mut rng = Rng::new(seed);
            let mut deals = 1;
            while !reasons_out(&super::lay(&mut rng, first), first) {
                deals += 1;
                assert!(deals <= 40, "seed {seed} clicked at {first:?} will not deal a fair field");
            }
        }
    }
}
