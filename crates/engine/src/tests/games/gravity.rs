use std::collections::HashSet;

use super::{Body, COLUMNS, PLANS, ROOMS, ROWS, mark, out, spiked, start, step};
use crate::game::{LIMIT, Outcome, TICK_HZ};
use crate::input::{Action, Key};
use crate::playing::LOOK;
use crate::session::Session;

/// The game's limit, in ticks.
const TICKS: u32 = LIMIT as u32 * TICK_HZ;

const CHOICES: [(f32, bool); 6] =
    [(1.0, false), (1.0, true), (0.0, false), (0.0, true), (-1.0, false), (-1.0, true)];

#[test]
fn every_room_is_walled_top_and_bottom_and_open_where_you_come_in() {
    for (number, plan) in PLANS.iter().enumerate() {
        for line in plan.iter() {
            assert_eq!(line.len(), COLUMNS, "room {number} has a line of the wrong length");
            assert!(
                line.bytes().all(|point| b"#^v.".contains(&point)),
                "room {number} has something in it that is not a block, a spike, or air"
            );
        }
        assert!(plan[0].bytes().all(|point| point == b'#'), "room {number} has no ceiling");
        assert!(plan[ROWS - 1].bytes().all(|point| point == b'#'), "room {number} has no floor");
        // A player carries their height from one room into the next, so the
        // whole of the left wall has to be open, not only the top and bottom.
        for row in 1..ROWS - 1 {
            assert_eq!(
                mark(number, 0, row as i32),
                b'.',
                "room {number} cannot be walked into at row {row}"
            );
        }
    }
}

/// Play the ten rooms through, searching over when to walk and when to turn
/// over, one choice a look. The rooms run on from one another, so this is
/// one search and not ten: the way up a room is left in is the way up the next
/// one starts. What comes out is then played back through a real session,
/// which is what holds the keys the game reads to the moves the search made.
#[test]
fn the_ten_rooms_can_be_walked_through_one_after_another() {
    let route = search(CHOICES).expect("no way through the ten rooms was found");

    let mut session = Session::new("gravity", 0).expect("it is cataloged");
    let mut outcome = Outcome::Playing;
    let mut walking = 0.0;
    for (way, turn) in route {
        let mut actions = Vec::new();
        if way != walking {
            for (side, key) in [(-1.0, Key::Left), (1.0, Key::Right)] {
                if walking == side {
                    actions.push(Action::Up(key));
                }
                if way == side {
                    actions.push(Action::Down(key));
                }
            }
            walking = way;
        }
        if turn {
            actions.push(Action::Tap(Key::Space));
        }
        outcome = session.advance(&actions, LOOK);
    }
    assert_eq!(outcome, Outcome::Won, "the route the search found does not finish the game");
}

/// A key on the state, so that a search does not walk the same ground twice.
fn seen_as(room: usize, body: &Body) -> (usize, i32, i32, i32, bool, bool) {
    (
        room,
        body.at.0.round() as i32,
        body.at.1.round() as i32,
        (body.drop / 10.0).round() as i32,
        body.over,
        body.standing,
    )
}

/// The fewest choices that finish the ten rooms, if there are any.
fn search(choices: [(f32, bool); 6]) -> Option<Vec<(f32, bool)>> {
    let mut frontier = vec![(0usize, start(), Vec::new())];
    let mut seen: HashSet<_> =
        frontier.iter().map(|(room, body, _)| seen_as(*room, body)).collect();
    let mut tick = 0;
    while tick < TICKS {
        let mut next = Vec::new();
        for (room, body, path) in &frontier {
            for (way, turn) in choices {
                let (mut room, mut body) = (*room, *body);
                let mut alive = true;
                let mut path = path.clone();
                path.push((way, turn));
                for held in 0..LOOK {
                    body = step(room, body, way, turn && held == 0);
                    if spiked(room, body.at) {
                        alive = false;
                        break;
                    }
                    if out(&body) {
                        if room + 1 == ROOMS {
                            return Some(path);
                        }
                        room += 1;
                        body.at.0 = super::HALF;
                    }
                }
                if alive && seen.insert(seen_as(room, &body)) {
                    next.push((room, body, path));
                }
            }
        }
        assert!(!next.is_empty(), "the search ran out of moves at tick {tick}");
        frontier = next;
        tick += LOOK;
    }
    None
}
