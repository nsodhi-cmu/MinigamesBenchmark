use super::{DRIFT, Ghost, PACE, solid};
use crate::game::{Game, Outcome, TICK};
use crate::input::{Action, Input, Key};
use crate::playing::LOOK;

/// The eight ways to skate, as the keys held and the way they carry you.
const WAYS: [(&[Key], (f32, f32)); 8] = [
    (&[Key::Left], (-1.0, 0.0)),
    (&[Key::Right], (1.0, 0.0)),
    (&[Key::Up], (0.0, -1.0)),
    (&[Key::Down], (0.0, 1.0)),
    (&[Key::Left, Key::Up], (-0.707, -0.707)),
    (&[Key::Right, Key::Up], (0.707, -0.707)),
    (&[Key::Left, Key::Down], (-0.707, 0.707)),
    (&[Key::Right, Key::Down], (0.707, 0.707)),
];

/// How far the player would be from the ghost after skating `looks` looks the
/// same way, worked out without the game so that the choice is the bot's own.
fn worth(from: (f32, f32), ghost: (f32, f32), way: (f32, f32), looks: u32) -> f32 {
    let (mut at, mut ghost) = (from, ghost);
    let mut nearest = f32::MAX;
    for _ in 0..looks * LOOK {
        let step = (way.0 * PACE * TICK, way.1 * PACE * TICK);
        for next in [(at.0 + step.0, at.1 + step.1), (at.0 + step.0, at.1), (at.0, at.1 + step.1)] {
            if !solid(next) {
                at = next;
                break;
            }
        }
        let (dx, dy) = (at.0 - ghost.0, at.1 - ghost.1);
        let gap = (dx * dx + dy * dy).sqrt().max(0.001);
        ghost = (ghost.0 + dx / gap * DRIFT * TICK, ghost.1 + dy / gap * DRIFT * TICK);
        nearest = nearest.min(gap);
    }
    nearest
}

/// A player who skates whichever way keeps the ghost furthest off over the next
/// second and a half lasts the whole round, so the park has no corner that
/// cannot be skated out of in time.
#[test]
fn a_player_who_looks_ahead_a_second_keeps_away_for_the_whole_round() {
    let mut game = Ghost::new();
    let mut outcome = Outcome::Playing;
    let mut ticks = 0;
    while outcome == Outcome::Playing {
        let best = WAYS
            .iter()
            .max_by(|a, b| {
                let (one, two) =
                    (worth(game.at, game.ghost, a.1, 8), worth(game.at, game.ghost, b.1, 8));
                one.total_cmp(&two)
            })
            .expect("there is always a way to go");
        let mut input = Input::new();
        for key in best.0 {
            input.apply(Action::Down(*key));
        }
        for _ in 0..LOOK {
            outcome = game.update(&input);
            ticks += 1;
            if outcome != Outcome::Playing {
                break;
            }
        }
    }
    assert_eq!(outcome, Outcome::Won, "caught after {ticks} ticks");
}
