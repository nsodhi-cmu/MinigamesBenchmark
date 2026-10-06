use super::{CAVES, Cave, Flier, PERIOD, Pyro};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input, Key};
use crate::playing::{LOOK, Went, fewest};

/// Which way across and whether UP is held, for one look.
const MOVES: [(i8, bool); 6] =
    [(0, false), (-1, false), (1, false), (0, true), (-1, true), (1, true)];

/// The keys of each look of the fewest looks through a cave, for a player
/// holding the same keys for a look at a time. Places a few pixels apart at the
/// same speed and the same moment of the bars' turn count as one.
fn crossing(c: &Cave) -> Option<Vec<usize>> {
    let start = Flier { at: c.find(b'S'), climb: 0.0 };
    let route = fewest(
        start,
        MOVES.len(),
        |f: Flier, tick| {
            (
                (f.at.0 / 6.0) as i32,
                (f.at.1 / 6.0) as i32,
                (f.climb / 30.0).round() as i32,
                tick % PERIOD,
            )
        },
        |f, tick, which| {
            let (across, up) = MOVES[which];
            let f = f.flown(c, f32::from(across), up);
            if c.out(f.at) {
                Went::Out(f)
            } else if c.burned(f.at, tick + 1) {
                Went::Dead
            } else {
                Went::On(f)
            }
        },
    )?;
    Some(route.into_iter().map(|(which, _)| which).collect())
}

/// A player who holds the same keys for a tenth of a second at a time can fly
/// through all three caves, and doing so through the game wins.
#[test]
fn a_player_who_looks_ten_times_a_second_flies_through_all_three_caves() {
    let mut game = Pyro::new();
    let mut outcome = Outcome::Playing;
    for (cave, map) in CAVES.iter().enumerate() {
        let route = crossing(map).expect("the cave can be flown through");
        assert_eq!(game.cave, cave);
        'look: for which in route {
            let (across, up) = MOVES[which];
            let mut input = Input::new();
            match across {
                -1 => input.apply(Action::Down(Key::Left)),
                1 => input.apply(Action::Down(Key::Right)),
                _ => {}
            }
            if up {
                input.apply(Action::Down(Key::Up));
            }
            for _ in 0..LOOK {
                outcome = game.update(&input);
                if outcome != Outcome::Playing || game.cave != cave {
                    break 'look;
                }
            }
        }
        assert_ne!(outcome, Outcome::Lost, "burned in cave {}", cave + 1);
    }
    assert_eq!(outcome, Outcome::Won);
}
