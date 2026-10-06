use super::{BUCKET_HALF, Fruits};
use crate::game::{Game, LIMIT, Outcome, TICK_HZ};
use crate::input::{Action, Input, Key};

const TICKS: u32 = LIMIT as u32 * TICK_HZ;

/// Plays out the whole limit holding whichever arrow `steer` picks each tick.
fn play(seed: u64, mut steer: impl FnMut(&Fruits) -> Option<Key>) -> (Outcome, u32, usize) {
    let mut game = Fruits::new(seed);
    for tick in 1..=TICKS {
        let mut input = Input::new();
        if let Some(key) = steer(&game) {
            input.apply(Action::Down(key));
        }
        let outcome = game.update(&input);
        if outcome != Outcome::Playing {
            return (outcome, tick, game.caught);
        }
    }
    (Outcome::Playing, TICKS, game.caught)
}

/// Heads for the fruit landing soonest that the bucket can still get under.
fn chase(game: &Fruits) -> Option<Key> {
    let reach = |fruit: &super::Fruit| {
        let left = (fruit.fall - fruit.age) as f32 / TICK_HZ as f32;
        (fruit.land - game.bucket).abs() <= super::BUCKET_PACE * left + BUCKET_HALF * 0.5
    };
    let target = game
        .fruits
        .iter()
        .filter(|fruit| fruit.age < fruit.fall && reach(fruit))
        .min_by_key(|fruit| fruit.fall - fruit.age)?;
    let gap = target.land - game.bucket;
    if gap.abs() < 4.0 {
        None
    } else if gap > 0.0 {
        Some(Key::Right)
    } else {
        Some(Key::Left)
    }
}

#[test]
fn a_player_chasing_the_next_fruit_to_land_meets_the_quota() {
    for seed in 0..8 {
        let (outcome, _, caught) = play(seed, chase);
        assert_eq!(outcome, Outcome::Won, "seed {seed} caught {caught}");
    }
}

/// A fruit says where it will land the moment it appears, and a swaying one
/// has to arrive there all the same: the sway vanishes at the end only because
/// the number of waves is whole. That is what a player is aiming by.
#[test]
fn a_swaying_fruit_arrives_where_it_said_it_would() {
    let mut game = Fruits::new(3);
    let idle = Input::new();
    let mut swayed = 0;
    for _ in 0..60 * TICK_HZ {
        game.update(&idle);
        for fruit in &game.fruits {
            if fruit.age == fruit.fall {
                assert!(
                    (fruit.at().0 - fruit.land).abs() < 0.5,
                    "a fruit meant for {} came down at {}",
                    fruit.land,
                    fruit.at().0
                );
                swayed += usize::from(fruit.sway > 0.0);
            }
        }
    }
    assert!(swayed > 0, "no swaying fruit was watched all the way down");
}
