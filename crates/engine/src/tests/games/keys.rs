use super::{KEYS, Kept, SHUFFLE};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

/// Shuffle a round all the way through, handing back the game the moment it
/// waits for a click.
fn shuffled(seed: u64) -> Kept {
    let mut game = Kept::new(seed);
    let input = Input::new();
    for _ in 0..SHUFFLE * 2 {
        assert_eq!(game.update(&input), Outcome::Playing, "the shuffling ended the round");
        if game.picking {
            return game;
        }
    }
    panic!("seed {seed} never stopped shuffling");
}

fn click(game: &mut Kept, at: (f32, f32)) -> Outcome {
    let mut input = Input::new();
    input.apply(Action::Press { button: Button::Left, x: at.0, y: at.1 });
    game.update(&input)
}

#[test]
fn clicking_the_key_that_glowed_wins_and_any_other_one_loses() {
    for seed in 0..12u64 {
        let mut game = shuffled(seed);
        let marked = game.marked;
        let other = (marked + 1 + (seed as usize % (KEYS - 1))) % KEYS;
        let mut wrong = shuffled(seed);
        let elsewhere = wrong.place(other);
        assert_eq!(click(&mut wrong, elsewhere), Outcome::Lost, "seed {seed} forgave a wrong key");
        let right = game.place(marked);
        assert_eq!(click(&mut game, right), Outcome::Won, "seed {seed} was not won");
    }
}

/// No two keys ever come near enough to be taken for one another, so a player
/// who watches carefully is never asked to guess.
#[test]
fn no_two_keys_ever_come_close_enough_to_be_confused() {
    for seed in 0..8u64 {
        let mut game = Kept::new(seed);
        let input = Input::new();
        let mut closest = f32::MAX;
        while !game.picking {
            game.update(&input);
            for one in 0..KEYS {
                for two in one + 1..KEYS {
                    let (a, b) = (game.place(one), game.place(two));
                    closest = closest.min(libm::hypotf(a.0 - b.0, a.1 - b.1));
                }
            }
        }
        assert!(closest > 34.0, "seed {seed} brings two keys within {closest:.1}");
    }
}
