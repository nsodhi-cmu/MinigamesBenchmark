use super::{CELL, Duo, GATES, HALF, ROOMS, Room};
use crate::game::{Game, LIMIT, Outcome, TICK_HZ};
use crate::input::{Action, Input, Key};
use crate::walk::Hopper;

/// The cells each square makes for in turn, until both are there. A room's
/// list ends on the two of them meeting.
type Leg = (Option<(usize, usize)>, Option<(usize, usize)>);

const ROUTES: [&[Leg]; 3] = [
    &[
        (Some((7, 18)), Some((25, 21))),
        (Some((10, 15)), Some((27, 18))),
        (Some((7, 12)), Some((30, 15))),
        (Some((9, 9)), Some((27, 12))),
        (Some((3, 6)), None),
        (None, Some((24, 12))),
        (None, Some((22, 9))),
        (None, Some((18, 6))),
        (Some((1, 21)), None),
        (Some((3, 21)), None),
        (Some((7, 18)), None),
        (Some((10, 15)), None),
        (Some((13, 21)), None),
        (Some((17, 21)), None),
        (None, Some((20, 12))),
        (None, Some((17, 21))),
    ],
    &[
        (Some((3, 21)), Some((28, 21))),
        (Some((6, 18)), Some((25, 18))),
        (Some((4, 15)), Some((27, 15))),
        (Some((7, 12)), Some((24, 12))),
        (Some((4, 9)), Some((27, 9))),
        (Some((9, 6)), Some((23, 6))),
        (Some((12, 21)), None),
        (Some((13, 20)), None),
        (None, Some((26, 21))),
        (None, Some((19, 21))),
        (None, Some((13, 20))),
    ],
    &[
        (None, Some((28, 7))),
        (Some((18, 21)), None),
        (Some((27, 18)), None),
        (Some((22, 15)), None),
        (Some((18, 13)), None),
        (None, Some((12, 9))),
        (None, Some((2, 13))),
        (None, Some((12, 13))),
        (Some((12, 13)), None),
    ],
];

/// What a square is told this tick to make for a cell: run toward it, and jump
/// when it is above or something is in the way.
fn steer(body: Hopper, room: Room, open: [bool; 3], goal: (usize, usize)) -> (f32, bool) {
    let (gx, gy) = room.map.board.middle(goal.0, goal.1);
    let across = if (gx - body.at.0).abs() > 3.0 { (gx - body.at.0).signum() } else { 0.0 };
    let ahead = room.blocked((body.at.0 + across * (HALF + 4.0), body.at.1), open);
    (across, gy < body.at.1 - CELL * 0.5 || (across != 0.0 && ahead))
}

fn arrived(body: Hopper, room: Room, open: [bool; 3], goal: (usize, usize)) -> bool {
    let (gx, gy) = room.map.board.middle(goal.0, goal.1);
    body.standing(|at| room.blocked(at, open))
        && (gx - body.at.0).abs() <= 4.0
        && (gy - body.at.1).abs() <= CELL * 0.6
}

/// Both squares driven leg by leg through every room, meeting at the end of
/// each, wins the game.
#[test]
fn two_squares_driven_along_their_routes_meet_in_every_room_and_win() {
    let mut game = Duo::new();
    let mut ticks = 0;
    for (room_at, legs) in ROUTES.iter().enumerate() {
        assert_eq!(game.room, room_at);
        let room = Room::of(&ROOMS[room_at]);
        for (leg, (blue_goal, red_goal)) in legs.iter().enumerate() {
            loop {
                let there = |body: Hopper, goal: Option<(usize, usize)>| {
                    goal.is_none_or(|goal| arrived(body, room, game.open(), goal))
                };
                if there(game.blue, *blue_goal) && there(game.red, *red_goal) {
                    break;
                }
                let mut input = Input::new();
                let keys = [(Key::A, Key::D, Key::W), (Key::Left, Key::Right, Key::Up)];
                for ((body, goal), (left, right, up)) in
                    [(game.blue, *blue_goal), (game.red, *red_goal)].into_iter().zip(keys)
                {
                    let Some(goal) = goal else { continue };
                    let (across, jump) = steer(body, room, game.open(), goal);
                    if across < 0.0 {
                        input.apply(Action::Down(left));
                    } else if across > 0.0 {
                        input.apply(Action::Down(right));
                    }
                    if jump {
                        input.apply(Action::Down(up));
                    }
                }
                let outcome = game.update(&input);
                ticks += 1;
                assert!(
                    ticks < LIMIT as u32 * TICK_HZ,
                    "room {} leg {leg} ran past the limit: blue at {:?}, red at {:?}",
                    room_at + 1,
                    game.blue.at,
                    game.red.at
                );
                if game.room != room_at || outcome == Outcome::Won {
                    assert_eq!(leg + 1, legs.len(), "room {} was won early", room_at + 1);
                    if outcome == Outcome::Won {
                        return;
                    }
                    break;
                }
            }
        }
    }
    panic!("the third room was never won");
}

/// A gate is open on the tick its plate is stood on and shut on the tick it is
/// left, and a square inside it then is lost.
#[test]
fn a_gate_shuts_the_tick_its_plate_is_left_and_crushes_a_square_inside_it() {
    let mut game = Duo::new();
    let room = Room::of(&ROOMS[0]);
    let (plate, gate) = GATES[0];
    let (px, py) = room.map.find(plate);
    game.blue = Hopper { at: (px, py - CELL * 0.5 - HALF), climb: 0.0 };
    game.red = Hopper { at: room.map.find(gate), climb: 0.0 };
    assert!(game.open()[0]);
    assert_eq!(game.update(&Input::new()), Outcome::Playing);
    game.blue = Hopper { at: room.map.find(b'1'), climb: 0.0 };
    assert_eq!(game.update(&Input::new()), Outcome::Lost);
}
