use super::{Phase, ROUNDS, Reflexes, WINDOW};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

fn wait_for_green(game: &mut Reflexes, input: &mut Input) -> Outcome {
    for _ in 0..1000 {
        let outcome = game.update(input);
        input.settle();
        if outcome != Outcome::Playing || matches!(game.phase, Phase::Flashed { .. }) {
            return outcome;
        }
    }
    panic!("the panel never turned green");
}

fn click(game: &mut Reflexes, input: &mut Input) -> Outcome {
    let (x, y) = input.cursor();
    input.apply(Action::Press { button: Button::Left, x, y });
    let outcome = game.update(input);
    input.settle();
    input.apply(Action::Release { button: Button::Left, x, y });
    input.settle();
    outcome
}

fn settle(game: &mut Reflexes, input: &mut Input) -> Outcome {
    let mut outcome = Outcome::Playing;
    for _ in 0..200 {
        outcome = game.update(input);
        input.settle();
        if outcome != Outcome::Playing || matches!(game.phase, Phase::Waiting { .. }) {
            break;
        }
    }
    outcome
}

#[test]
fn clicking_the_moment_it_turns_carries_every_round() {
    for seed in 0..8u64 {
        let mut game = Reflexes::new(seed);
        let mut input = Input::new();
        let mut outcome = Outcome::Playing;
        for round in 0..ROUNDS {
            wait_for_green(&mut game, &mut input);
            outcome = click(&mut game, &mut input);
            assert_eq!(outcome, Outcome::Playing, "seed {seed} round {round} was refused");
            outcome = settle(&mut game, &mut input);
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} did not finish");
    }
}

/// Clicking before it turns is the whole reason the game cannot be beaten by
/// clicking every frame.
#[test]
fn clicking_before_it_turns_ends_the_round() {
    let mut game = Reflexes::new(1);
    let mut input = Input::new();
    assert_eq!(click(&mut game, &mut input), Outcome::Lost, "an early click was allowed");
}

#[test]
fn letting_the_half_second_run_out_ends_the_round() {
    let mut game = Reflexes::new(2);
    let mut input = Input::new();
    wait_for_green(&mut game, &mut input);
    let mut outcome = Outcome::Playing;
    for _ in 0..WINDOW + 2 {
        outcome = game.update(&input);
        input.settle();
        if outcome != Outcome::Playing {
            break;
        }
    }
    assert_eq!(outcome, Outcome::Lost, "waiting the window out was allowed");
}
