use super::{BURST, Balloon, NEAR, PUFF, START, TARGET};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input, Key};
use crate::playing::LOOK;

fn tap(game: &mut Balloon, key: Key) -> Outcome {
    let mut input = Input::new();
    input.apply(Action::Tap(key));
    let outcome = game.update(&input);
    let idle = Input::new();
    for _ in 1..LOOK {
        if outcome != Outcome::Playing {
            return outcome;
        }
        game.update(&idle);
    }
    outcome
}

/// One puff each look climbs into the window about the dashed circle rather
/// than over it, and launching there wins.
#[test]
fn puffing_once_a_look_up_to_the_line_and_launching_wins() {
    const { assert!(PUFF < NEAR * 2.0, "one puff can step clean over the window") };
    let mut game = Balloon::new();
    while game.radius < TARGET - NEAR {
        assert_eq!(tap(&mut game, Key::Space), Outcome::Playing);
    }
    assert!(game.radius <= TARGET + NEAR);
    assert_eq!(tap(&mut game, Key::Up), Outcome::Won);
}

#[test]
fn launching_short_loses_and_puffing_on_bursts_it() {
    let mut short = Balloon::new();
    for _ in 0..8 {
        tap(&mut short, Key::Space);
    }
    assert_eq!(tap(&mut short, Key::Up), Outcome::Lost);

    let mut game = Balloon::new();
    let mut puffs = 0;
    while tap(&mut game, Key::Space) == Outcome::Playing {
        puffs += 1;
        assert!(START + puffs as f32 * PUFF < BURST * 2.0, "the balloon never burst");
    }
    assert!(game.radius >= BURST);
}
