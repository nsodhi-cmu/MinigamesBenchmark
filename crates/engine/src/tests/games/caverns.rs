use super::{CAVERNS, Cavern, Caverns, Diver, ROWS, Step};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input, Key};
use crate::playing::{LOOK, Went, fewest};

/// Which way across and which way up, for one look.
const MOVES: [(i8, i8); 9] =
    [(0, 0), (-1, 0), (1, 0), (0, 1), (0, -1), (-1, 1), (1, 1), (-1, -1), (1, -1)];

/// The keys of each look of the fewest looks through a cavern. Places a few
/// pixels apart with about as much breath left count as one.
fn crossing(cavern: Cavern) -> Option<Vec<usize>> {
    let start = Diver { at: cavern.find(b'S'), fall: 0.0, breath: 1.0 };
    let route = fewest(
        start,
        MOVES.len(),
        |d: Diver, _| {
            (
                (d.at.0 / 5.0) as i32,
                (d.at.1 / 5.0) as i32,
                (d.breath * 25.0) as i32,
                (d.fall / 80.0) as i32,
            )
        },
        |d, tick, which| {
            let (across, up) = MOVES[which];
            // A kick is a press, so it lands on the first tick of a look only.
            let kick = up > 0 && tick % LOOK == 0;
            match d.step(cavern, f32::from(across), kick, up < 0) {
                Step::Moved(d) => Went::On(d),
                Step::Out => Went::Out(d),
                Step::Drowned => Went::Dead,
            }
        },
    )?;
    Some(route.into_iter().map(|(which, _)| which).collect())
}

/// A player who holds the same keys for a tenth of a second at a time can swim
/// through all three caverns, and doing so through the game wins.
#[test]
fn a_player_who_looks_ten_times_a_second_swims_through_all_three_caverns() {
    let mut game = Caverns::new();
    let mut outcome = Outcome::Playing;
    for (cavern, map) in CAVERNS.iter().enumerate() {
        assert_eq!(game.cavern, cavern);
        let route = crossing(Cavern::of(map)).expect("the cavern can be swum");
        'look: for which in route {
            let (across, up) = MOVES[which];
            let mut input = Input::new();
            match across {
                -1 => input.apply(Action::Down(Key::Left)),
                1 => input.apply(Action::Down(Key::Right)),
                _ => {}
            }
            if up < 0 {
                input.apply(Action::Down(Key::Down));
            }
            for tick in 0..LOOK {
                if up > 0 && tick == 0 {
                    input.apply(Action::Tap(Key::Up));
                } else if up > 0 && tick == 1 {
                    input.settle();
                }
                outcome = game.update(&input);
                if outcome != Outcome::Playing || game.cavern != cavern {
                    break 'look;
                }
            }
        }
        assert_ne!(outcome, Outcome::Lost, "drowned in cavern {}", cavern + 1);
    }
    assert_eq!(outcome, Outcome::Won);
}

/// Air stays where rock holds it: every pocket is roofed and walled in on both
/// sides, so none of it is left sitting against open water.
#[test]
fn every_pocket_of_air_is_held_in_by_rock() {
    for (cavern, map) in CAVERNS.iter().enumerate() {
        for row in 3..ROWS {
            let line = map[row].as_bytes();
            for (column, mark) in line.iter().enumerate() {
                if *mark != b'a' {
                    continue;
                }
                let holds = |mark: u8| mark == b'#' || mark == b'a';
                assert!(
                    holds(line[column - 1]) && holds(line[column + 1]),
                    "cavern {} has air spilling sideways at {column}, {row}",
                    cavern + 1
                );
                assert!(
                    holds(map[row - 1].as_bytes()[column]),
                    "cavern {} has air with nothing over it at {column}, {row}",
                    cavern + 1
                );
            }
        }
    }
}
