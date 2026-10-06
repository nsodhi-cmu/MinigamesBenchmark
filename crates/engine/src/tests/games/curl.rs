use super::{Curl, HACK, PULL, STONES, on_house};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

/// Throw one stone at the speed asked for, sweeping ahead of it or not, and
/// let the ice take it wherever it goes.
fn cast(game: &mut Curl, pace: (f32, f32), sweeping: bool) -> Outcome {
    let mut input = Input::new();
    input.apply(Action::Press { button: Button::Left, x: HACK.0, y: HACK.1 });
    game.update(&input);
    input.settle();
    input.apply(Action::Move { x: HACK.0 - pace.0 / PULL, y: HACK.1 - pace.1 / PULL });
    game.update(&input);
    input.settle();
    let (x, y) = input.cursor();
    input.apply(Action::Release { button: Button::Left, x, y });
    let mut outcome = game.update(&input);
    input.settle();

    for tick in 0..60 * 30 {
        if sweeping && tick % 12 == 0 {
            let (x, y) = input.cursor();
            input.apply(Action::Click { button: Button::Left, x, y });
        }
        outcome = game.update(&input);
        input.settle();
        if outcome != Outcome::Playing || game.stones.iter().all(|s| s.pace.0 == 0.0) {
            break;
        }
    }
    outcome
}

/// A takeout down the middle followed by two draws on either side of it clears
/// the red stone off the end and leaves all three of ours in the house.
#[test]
fn a_takeout_and_two_draws_finishes_the_round() {
    let mut game = Curl::new();
    assert_eq!(cast(&mut game, (330.0, 0.0), false), Outcome::Playing);
    assert!(game.stones.iter().all(|stone| stone.mine), "the takeout left the red stone in play");
    assert_eq!(cast(&mut game, (161.9, -20.5), false), Outcome::Playing);
    assert_eq!(cast(&mut game, (161.9, 20.5), false), Outcome::Won, "the round was not won");
    assert_eq!(
        game.stones.iter().filter(|stone| on_house(stone.at)).count(),
        STONES,
        "the three draws did not all stay in the house"
    );
}

/// Three gentle draws that never reach the red stone leave it sitting on the
/// house, which loses however well they are placed.
#[test]
fn leaving_the_red_stone_where_it_lies_loses_the_round() {
    let mut game = Curl::new();
    cast(&mut game, (161.9, -20.5), false);
    cast(&mut game, (161.9, 20.5), false);
    let outcome = cast(&mut game, (130.0, 0.0), false);
    assert_eq!(outcome, Outcome::Lost, "the red stone was left in the house and it was allowed");
}

/// Sweeping cuts the friction, so the same throw run over swept ice finishes
/// farther down the sheet than it would on bare ice.
#[test]
fn sweeping_carries_a_stone_further_than_bare_ice_would() {
    let pace = (140.0, 0.0);
    let mut bare = Curl::new();
    cast(&mut bare, pace, false);
    let mut swept = Curl::new();
    cast(&mut swept, pace, true);
    let far =
        |game: &Curl| game.stones.iter().find(|stone| stone.mine).map_or(0.0, |stone| stone.at.0);
    assert!(
        far(&swept) > far(&bare) + 20.0,
        "sweeping carried the stone to {:.0} against {:.0} on bare ice",
        far(&swept),
        far(&bare)
    );
}

/// A stone thrown hard enough runs off the far end of the sheet and is gone,
/// which is how the red one is got rid of.
#[test]
fn a_stone_thrown_off_the_end_is_out_of_play() {
    let mut game = Curl::new();
    cast(&mut game, (350.0, -90.0), false);
    assert_eq!(
        game.stones.iter().filter(|stone| stone.mine).count(),
        0,
        "a stone thrown past the back line stayed in play"
    );
    assert!(game.stones.iter().any(|stone| !stone.mine), "it took the red stone with it");
}
