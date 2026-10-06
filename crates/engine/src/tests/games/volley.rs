use super::{BALL, BAT_WIDE, BOTTOM, MINE, SURVIVE, TOP, Volley};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input, Key};

/// Where the ball will be when it reaches the near bat, by folding its path
/// back and forth between the roof and the floor. Anticipating this is what
/// the round asks for: the ball ends up faster than the bat can move.
fn arriving_at(volley: &Volley) -> f32 {
    let face = MINE + BAT_WIDE + BALL;
    if volley.drift.0 >= 0.0 {
        return volley.ball.1;
    }
    let (low, high) = (TOP + BALL, BOTTOM - BALL);
    let span = high - low;
    let reach = (volley.ball.0 - face) / -volley.drift.0;
    let mut along = (volley.ball.1 - low + volley.drift.1 * reach).rem_euclid(span * 2.0);
    if along > span {
        along = span * 2.0 - along;
    }
    low + along
}

#[test]
fn a_player_who_reads_where_the_ball_is_going_lasts_the_round() {
    for seed in 0..8u64 {
        let mut volley = Volley::new(seed);
        let mut outcome = Outcome::Playing;
        let mut ticks = 0;
        while outcome == Outcome::Playing {
            let wanted = arriving_at(&volley);
            let mut input = Input::new();
            if wanted > volley.mine + 1.0 {
                input.apply(Action::Down(Key::Down));
            } else if wanted < volley.mine - 1.0 {
                input.apply(Action::Down(Key::Up));
            }
            outcome = volley.update(&input);
            ticks += 1;
            assert!(ticks <= SURVIVE, "seed {seed} ran past the end of the round");
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} lost after {ticks} ticks");
    }
}
