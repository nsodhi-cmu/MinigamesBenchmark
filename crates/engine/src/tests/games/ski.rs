use super::{GATES, HALF, Ski};
use crate::game::{Game, Outcome};
use crate::input::Input;

/// The next tick carries the skier from just before gate `which` to just past it,
/// `sideways` posts' worth off its middle, going `forward` through it or not.
fn cross(game: &mut Ski, which: usize, forward: bool, sideways: f32) -> Outcome {
    let gate = game.gates[which];
    let sign = if forward { 1.0 } else { -1.0 };
    let (ax, ay) = gate.across();
    game.at = (
        gate.at.0 - gate.way.0 * 2.0 * sign + ax * sideways,
        gate.at.1 - gate.way.1 * 2.0 * sign + ay * sideways,
    );
    game.pace = (gate.way.0 * 300.0 * sign, gate.way.1 * 300.0 * sign);
    game.update(&Input::new())
}

#[test]
fn six_gates_in_order_the_way_the_arrows_point_wins_on_the_sixth() {
    let mut game = Ski::new(4);
    for which in 0..GATES {
        let outcome = cross(&mut game, which, true, HALF - 1.0);
        assert_eq!(outcome, if which + 1 == GATES { Outcome::Won } else { Outcome::Playing });
        assert_eq!(game.passed, which + 1);
    }
}

#[test]
fn a_gate_taken_backwards_out_of_turn_or_again_ends_the_round() {
    let mut backwards = Ski::new(4);
    assert_eq!(cross(&mut backwards, 0, false, 0.0), Outcome::Lost);

    let mut early = Ski::new(4);
    assert_eq!(cross(&mut early, 1, true, 0.0), Outcome::Lost);

    let mut again = Ski::new(4);
    assert_eq!(cross(&mut again, 0, true, 0.0), Outcome::Playing);
    assert_eq!(cross(&mut again, 0, true, 0.0), Outcome::Lost);
}

/// Passing just outside a post is passing nothing at all.
#[test]
fn going_round_the_outside_of_a_post_is_not_going_through() {
    let mut game = Ski::new(4);
    assert_eq!(cross(&mut game, 2, false, HALF + 1.0), Outcome::Playing);
    assert_eq!(game.passed, 0);
}
