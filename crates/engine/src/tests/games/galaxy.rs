use super::{DRONES, Galaxy};
use crate::game::{Game, LIMIT, Outcome, TICK_HZ};
use crate::input::{Action, Input, Key};
use crate::playing::{LOOK, holding};

/// Whether holding `keys` for one look and then standing still for a second
/// and a half keeps the ship clear of every drone.
fn safe(game: &Galaxy, keys: &[Key]) -> bool {
    let mut game = game.clone();
    let (moving, still) = (holding(keys), Input::new());
    (0..TICK_HZ * 3 / 2).all(|tick| {
        let input = if tick < LOOK { &moving } else { &still };
        game.update(input) != Outcome::Lost
    })
}

/// Whether a shot fired now, holding `keys` for the look, brings a drone down.
fn lands(game: &Galaxy, keys: &[Key]) -> bool {
    let mut game = game.clone();
    let mut input = holding(keys);
    input.apply(Action::Tap(Key::Space));
    let down = game.down();
    for _ in 0..TICK_HZ {
        if game.update(&input) == Outcome::Won {
            return true;
        }
        input.settle();
        if game.down() > down {
            return true;
        }
    }
    false
}

/// A player who looks ten times a second, keeps clear of the drones, and fires
/// when a shot would land, shoots all twenty down inside the limit.
#[test]
fn a_player_who_keeps_clear_and_fires_when_a_shot_would_land_clears_the_formation() {
    let mut game = Galaxy::new();
    let mut looks = 0;
    let mut outcome = Outcome::Playing;
    while outcome == Outcome::Playing {
        looks += 1;
        assert!(
            (looks * LOOK) as f32 / (TICK_HZ as f32) < LIMIT,
            "the formation was never cleared"
        );
        let target = (0..DRONES)
            .filter_map(|which| game.position(which))
            .map(|(x, _)| x)
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
        if lands(&game, keys) {
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
    assert_eq!(outcome, Outcome::Won, "{} left after {looks} looks", DRONES - game.down());
}
