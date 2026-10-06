use super::{Hue, NEAR, REVEAL, ROUNDS, SHOW, Stage, TRACK_W, TRACK_X, TRACK_Y, apart, hsv};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input, Key};

fn run(game: &mut Hue, actions: &[Action], ticks: u32) -> Outcome {
    let mut input = Input::new();
    for action in actions {
        input.apply(*action);
    }
    let mut outcome = game.update(&input);
    input.settle();
    for _ in 1..ticks {
        if outcome != Outcome::Playing {
            break;
        }
        outcome = game.update(&input);
    }
    outcome
}

/// Clicking each track where the color shown would put it, once the color is
/// gone, mixes it back within the tolerance three times over.
#[test]
fn mixing_the_color_shown_after_it_is_gone_wins_in_three_rounds() {
    let mut game = Hue::new(5);
    for round in 0..ROUNDS {
        assert_eq!(run(&mut game, &[Action::Tap(Key::Enter)], SHOW), Outcome::Playing);
        assert!(matches!(game.stage, Stage::Mixing), "enter during the showing counted");
        for (which, &y) in TRACK_Y.iter().enumerate() {
            let x = TRACK_X + game.target[which] * TRACK_W;
            run(&mut game, &[Action::Click { button: Button::Left, x, y }], 1);
        }
        assert!(apart(hsv(game.mix), hsv(game.target)) <= NEAR);
        let outcome = run(&mut game, &[Action::Tap(Key::Enter)], REVEAL + 1);
        assert_eq!(outcome, if round + 1 == ROUNDS { Outcome::Won } else { Outcome::Playing });
    }
}

/// The verdict comes only once the full second side by side is over, and sliders left
/// where they started are never near enough.
#[test]
fn a_color_left_unmixed_loses_once_the_reveal_has_run() {
    let mut game = Hue::new(11);
    run(&mut game, &[Action::Wait], SHOW);
    assert_eq!(run(&mut game, &[Action::Tap(Key::Enter)], REVEAL), Outcome::Playing);
    assert_eq!(run(&mut game, &[Action::Wait], 1), Outcome::Lost);
}
