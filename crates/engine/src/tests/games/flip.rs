use super::{Flip, PHASE, ROOMS, Room, Step, step};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input, Key};
use crate::playing::{LOOK, Went, fewest};
use crate::walk::Hopper;

/// Which way across and whether jump is held, for one look.
const MOVES: [(i8, bool); 6] =
    [(0, false), (-1, false), (1, false), (0, true), (-1, true), (1, true)];

/// The keys of each look of the fewest looks through a room, for a player
/// holding the same keys for a look at a time. Places a few pixels apart at
/// the same speed and the same moment of the swaps count as one.
fn crossing(room: Room) -> Option<Vec<usize>> {
    let start = Hopper { at: room.find(b'S'), climb: 0.0 };
    let route = fewest(
        start,
        MOVES.len(),
        |h: Hopper, tick| {
            (
                (h.at.0 / 4.0) as i32,
                (h.at.1 / 4.0) as i32,
                (h.climb / 60.0).round() as i32,
                tick % (PHASE * 2),
            )
        },
        |h, tick, which| {
            let (across, jump) = MOVES[which];
            match step(h, room, tick + 1, f32::from(across), jump) {
                Step::Moved(h) => Went::On(h),
                Step::Out => Went::Out(h),
                Step::Crushed | Step::Fell => Went::Dead,
            }
        },
    )?;
    Some(route.into_iter().map(|(which, _)| which).collect())
}

/// A player who holds the same keys for a tenth of a second at a time can get
/// through all three rooms, and doing so through the game wins.
#[test]
fn a_player_who_looks_ten_times_a_second_gets_through_all_three_rooms() {
    let mut game = Flip::new();
    let mut outcome = Outcome::Playing;
    for (room, map) in ROOMS.iter().enumerate() {
        assert_eq!(game.room, room);
        let route = crossing(Room::of(map)).expect("the room can be crossed");
        'look: for which in route {
            let (across, jump) = MOVES[which];
            let mut input = Input::new();
            match across {
                -1 => input.apply(Action::Down(Key::Left)),
                1 => input.apply(Action::Down(Key::Right)),
                _ => {}
            }
            if jump {
                input.apply(Action::Down(Key::Up));
            }
            for _ in 0..LOOK {
                outcome = game.update(&input);
                if outcome != Outcome::Playing || game.room != room {
                    break 'look;
                }
            }
        }
        assert_ne!(outcome, Outcome::Lost, "lost in room {}", room + 1);
    }
    assert_eq!(outcome, Outcome::Won);
}
