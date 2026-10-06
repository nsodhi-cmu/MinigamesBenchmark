use super::{Kind, LASTS, RUNNER_X, Vault, WIDE};
use crate::game::{Game, Outcome, TICK};
use crate::input::{Action, Input, Key};

/// A player who jumps a block as it comes and ducks a bar as it comes. The
/// jump is asked for while the block is still far enough off that the runner
/// is over it by the time it arrives.
fn run(seed: u64) -> (Outcome, u32) {
    let mut game = Vault::new(seed);
    let mut outcome = Outcome::Playing;
    let mut ticks = 0;
    while outcome == Outcome::Playing {
        let shift = scrolled(&game);
        let next = game
            .hurdles
            .iter()
            .map(|hurdle| (hurdle.start - shift, &hurdle.kind))
            .filter(|(at, _)| at + WIDE > RUNNER_X - 13.0)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let mut input = Input::new();
        if let Some((at, kind)) = next {
            let away = at - (RUNNER_X + 13.0);
            match kind {
                Kind::Block(_) if (0.0..90.0).contains(&away) => {
                    input.apply(Action::Tap(Key::Space));
                }
                Kind::Bar if away < 60.0 => input.apply(Action::Down(Key::Down)),
                _ => {}
            }
        }
        outcome = game.update(&input);
        ticks += 1;
        assert!(ticks <= LASTS + 1, "seed {seed} ran past the end of the round");
    }
    (outcome, ticks)
}

fn scrolled(game: &Vault) -> f32 {
    super::SCROLL * (game.age as f32 * TICK)
}

#[test]
fn a_player_who_jumps_the_blocks_and_ducks_the_bars_lasts_the_round() {
    for seed in 0..12u64 {
        let (outcome, ticks) = run(seed);
        assert_eq!(outcome, Outcome::Won, "seed {seed} fell at tick {ticks}");
    }
}

#[test]
fn every_course_lays_out_its_hurdles_in_order() {
    for seed in 0..12u64 {
        for pair in Vault::new(seed).hurdles.windows(2) {
            assert!(pair[0].start < pair[1].start, "seed {seed} laid two hurdles out of order");
        }
    }
}
