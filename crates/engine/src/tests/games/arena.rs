use super::{Arena, MIDDLE};
use crate::game::{Game, Outcome, TICK_HZ};
use crate::input::{Action, Input, Key};
use crate::playing::LOOK;

/// The arrow keys held to push as nearly as they can along a direction.
fn keys_toward(way: (f32, f32)) -> Input {
    let mut input = Input::new();
    let length = libm::hypotf(way.0, way.1);
    if length < 1.0 {
        return input;
    }
    for (lean, minus, plus) in
        [(way.0 / length, Key::Left, Key::Right), (way.1 / length, Key::Up, Key::Down)]
    {
        if lean > 0.38 {
            input.apply(Action::Down(plus));
        } else if lean < -0.38 {
            input.apply(Action::Down(minus));
        }
    }
    input
}

/// Hold the middle of the arena, and charge whichever chaser comes within
/// `reach`. A chaser that comes for you there is always the one nearer the
/// wall, so a charge sends it outward.
fn hold_the_middle(reach: f32) -> (Outcome, u32) {
    let mut game = Arena::new();
    let mut ticks = 0;
    loop {
        let me = game.player;
        let nearest = game
            .chasers
            .iter()
            .flatten()
            .map(|body| (body.at, libm::hypotf(body.at.0 - me.at.0, body.at.1 - me.at.1)))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let way = match nearest {
            Some((at, far)) if far < reach => (at.0 - me.at.0, at.1 - me.at.1),
            _ => (MIDDLE.0 - me.at.0 - me.pace.0 * 1.2, MIDDLE.1 - me.at.1 - me.pace.1 * 1.2),
        };
        let input = keys_toward(way);
        for _ in 0..LOOK {
            ticks += 1;
            let outcome = game.update(&input);
            if outcome != Outcome::Playing || ticks >= 180 * TICK_HZ {
                return (outcome, ticks);
            }
        }
    }
}

/// A plain plan played ten looks a second clears the arena from at least one
/// of a spread of distances to charge from.
#[test]
fn holding_the_middle_and_charging_what_comes_close_clears_the_arena() {
    let tries: Vec<(Outcome, u32)> =
        (0..25).map(|step| hold_the_middle(50.0 + step as f32 * 5.0)).collect();
    assert!(tries.iter().any(|(outcome, _)| *outcome == Outcome::Won), "no try won: {tries:?}");
}
