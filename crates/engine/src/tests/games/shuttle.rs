use super::{SAUCER_WIDE, SAUCERS, Shuttle};
use crate::game::{Game, LIMIT, Outcome, TICK_HZ};
use crate::input::{Action, Input, Key};
use crate::playing::{LOOK, holding};

/// Whether holding `keys` for one look and then standing still for most of a
/// second gets the ship through everything already falling.
fn safe(game: &Shuttle, keys: &[Key]) -> bool {
    let mut game = game.clone();
    let (moving, still) = (holding(keys), Input::new());
    (0..TICK_HZ).all(|tick| {
        let input = if tick < LOOK { &moving } else { &still };
        game.update(input) != Outcome::Lost
    })
}

/// Whether a shot fired now, holding `keys` for the look, brings a saucer down.
fn lands(game: &Shuttle, keys: &[Key]) -> bool {
    let mut game = game.clone();
    let mut input = holding(keys);
    input.apply(Action::Tap(Key::Space));
    let standing = game.up;
    for _ in 0..TICK_HZ {
        game.update(&input);
        input.settle();
        if game.up != standing {
            return true;
        }
        if game.shot.is_none() {
            return false;
        }
    }
    false
}

/// A player who looks ten times a second, keeps out from under what is falling,
/// and fires when a saucer is above, clears the row inside the limit.
#[test]
fn a_player_who_dodges_and_fires_when_under_a_saucer_clears_the_row() {
    let mut game = Shuttle::new();
    let mut looks = 0;
    let mut outcome = Outcome::Playing;
    while outcome == Outcome::Playing {
        looks += 1;
        assert!((looks * LOOK) as f32 / (TICK_HZ as f32) < LIMIT, "the row was never cleared");
        let target = (0..SAUCERS)
            .filter(|which| game.up[*which])
            .map(|which| game.saucer_middle(which))
            .min_by(|a, b| (a - game.ship).abs().total_cmp(&(b - game.ship).abs()))
            .unwrap();
        let toward: &[Key] = if target > game.ship + 4.0 {
            &[Key::Right]
        } else if target < game.ship - 4.0 {
            &[Key::Left]
        } else {
            &[]
        };
        let choices: [&[Key]; 4] = [toward, &[], &[Key::Left], &[Key::Right]];
        let keys = choices.into_iter().find(|keys| safe(&game, keys)).unwrap_or(&[]);
        let mut input = holding(keys);
        if game.shot.is_none() && (target - game.ship).abs() < SAUCER_WIDE && lands(&game, keys) {
            input.apply(Action::Tap(Key::Space));
        }
        for _ in 0..LOOK {
            outcome = game.update(&input);
            input.settle();
            if outcome != Outcome::Playing {
                break;
            }
        }
    }
    assert_eq!(outcome, Outcome::Won, "row {:?} up {:?} looks {looks}", game.row, game.up);
}

/// The saucers' shots go where the ship is heading, so a ship that keeps
/// running the same way is hit rather than outrun.
#[test]
fn running_back_and_forth_without_firing_is_shot_down() {
    let mut game = Shuttle::new();
    let mut tick = 0;
    loop {
        let keys: &[Key] =
            if (tick / (2 * TICK_HZ)).is_multiple_of(2) { &[Key::Left] } else { &[Key::Right] };
        match game.update(&holding(keys)) {
            Outcome::Lost => break,
            Outcome::Won => panic!("won without firing"),
            Outcome::Playing => {}
        }
        tick += 1;
    }
    assert!(game.row.1 < 300.0, "the row landed before a shot found the ship");
}
