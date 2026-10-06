use super::{Clocked, Phase, TRIALS};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input, Key};

fn tap(game: &mut Clocked, input: &mut Input) -> Outcome {
    input.apply(Action::Down(Key::Space));
    let outcome = game.update(input);
    input.settle();
    input.apply(Action::Up(Key::Space));
    input.settle();
    outcome
}

/// Lets time pass the way a session does, which is to say it stops the moment
/// the round is over rather than going on updating a finished game.
fn idle(game: &mut Clocked, input: &mut Input, ticks: u32) -> Outcome {
    let mut outcome = Outcome::Playing;
    for _ in 0..ticks {
        outcome = game.update(input);
        input.settle();
        if outcome != Outcome::Playing {
            break;
        }
    }
    outcome
}

/// Counting the asked-for time out exactly passes every trial.
#[test]
fn stopping_on_the_asked_for_tick_carries_the_round() {
    for seed in 0..8u64 {
        let mut game = Clocked::new(seed);
        let mut input = Input::new();
        let mut outcome = Outcome::Playing;
        for trial in 0..TRIALS {
            let asked = game.asked;
            tap(&mut game, &mut input);
            assert!(matches!(game.phase, Phase::Counting { .. }), "the count did not start");
            idle(&mut game, &mut input, asked - 1);
            outcome = tap(&mut game, &mut input);
            assert_eq!(outcome, Outcome::Playing, "seed {seed} trial {trial} was called wrong");
            outcome = idle(&mut game, &mut input, 200);
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} did not finish");
    }
}

/// Stopping outside the tenth allowed does not.
#[test]
fn stopping_well_wide_of_it_does_not() {
    let mut game = Clocked::new(1);
    let mut input = Input::new();
    let asked = game.asked;
    tap(&mut game, &mut input);
    idle(&mut game, &mut input, asked / 2);
    tap(&mut game, &mut input);
    assert_eq!(idle(&mut game, &mut input, 200), Outcome::Lost, "half the time was accepted");
}
