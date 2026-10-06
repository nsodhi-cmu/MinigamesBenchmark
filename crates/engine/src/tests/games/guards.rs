use std::collections::VecDeque;

use super::{FADE, Guards, PACE, ROOMS, guard_at, open, sees};
use crate::game::{Game, Outcome, TICK};
use crate::input::{Action, Input, Key};
use crate::playing::LOOK;

/// How far one look of holding an arrow carries the player.
const STRIDE: f32 = PACE * TICK * LOOK as f32;
const MOVES: [Option<(Key, i32, i32)>; 5] = [
    None,
    Some((Key::Up, 0, -1)),
    Some((Key::Down, 0, 1)),
    Some((Key::Left, -1, 0)),
    Some((Key::Right, 1, 0)),
];
const ACROSS: i32 = 90;
const DOWN: i32 = 70;

/// The places a player holding one arrow a look can stand, as strides from the start.
fn spot(room: usize, across: i32, down: i32) -> (f32, f32) {
    let start = ROOMS[room].start;
    (start.0 + across as f32 * STRIDE, start.1 + down as f32 * STRIDE)
}

/// For every tick of the room's round and every spot, whether any guard's light
/// falls within a few pixels of it, which covers every point a stride passes.
fn lights(room: usize, near: f32) -> Vec<Vec<bool>> {
    let r = &ROOMS[room];
    (0..r.period)
        .map(|tick| {
            let guards: Vec<_> = r.patrols.iter().map(|p| guard_at(p, r.period, tick)).collect();
            let mut lit = vec![false; (ACROSS * 2 * DOWN * 2) as usize];
            for a in -ACROSS..ACROSS {
                for d in -DOWN..DOWN {
                    let (x, y) = spot(room, a, d);
                    lit[((a + ACROSS) * DOWN * 2 + d + DOWN) as usize] =
                        guards.iter().any(|(at, face)| {
                            [(0.0, 0.0), (near, 0.0), (-near, 0.0), (0.0, near), (0.0, -near)]
                                .iter()
                                .any(|(dx, dy)| sees(r, *at, *face, (x + dx, y + dy)))
                        });
                }
            }
            lit
        })
        .collect()
}

/// The fewest looks across a room holding one arrow or none a look, where the
/// player is never lit while they can be seen.
fn crossing(room: usize) -> Option<Vec<Option<Key>>> {
    let r = &ROOMS[room];
    let lit = lights(room, STRIDE * 0.6);
    let looks = r.period / LOOK;
    let fades = FADE / LOOK + 1;
    let index = |a: i32, d: i32, phase: u32, fade: u32| {
        ((((a + ACROSS) * DOWN * 2 + d + DOWN) as u32 * looks + phase) * fades + fade) as usize
    };
    let size = (ACROSS * 2 * DOWN * 2) as usize * (looks * fades) as usize;
    let mut came: Vec<Option<(usize, u8)>> = vec![None; size];
    let first = index(0, 0, 0, 0);
    came[first] = Some((first, 0));
    let mut queue = VecDeque::from([(0i32, 0i32, 0u32, 0u32)]);
    while let Some((a, d, phase, fade)) = queue.pop_front() {
        for (which, step) in MOVES.iter().enumerate() {
            let (na, nd) = match step {
                Some((_, da, dd)) => (a + da, d + dd),
                None => (a, d),
            };
            if !(-ACROSS..ACROSS).contains(&na) || !(-DOWN..DOWN).contains(&nd) {
                continue;
            }
            let (x, y) = spot(room, na, nd);
            let (mx, my) = spot(room, a, d);
            if !open(r, x, y) || !open(r, (x + mx) * 0.5, (y + my) * 0.5) {
                continue;
            }
            let mut safe = true;
            for tick in 1..=LOOK {
                let now = ((phase * LOOK + tick) % r.period) as usize;
                let seen_here = step.is_some() || fade * LOOK + tick < FADE;
                let cells = [
                    ((a + ACROSS) * DOWN * 2 + d + DOWN) as usize,
                    ((na + ACROSS) * DOWN * 2 + nd + DOWN) as usize,
                ];
                let shone = cells.iter().any(|c| lit[now][*c]);
                if shone && seen_here {
                    safe = false;
                }
            }
            if !safe {
                continue;
            }
            let nfade = if step.is_some() { 0 } else { (fade + 1).min(fades - 1) };
            let nphase = (phase + 1) % looks;
            let (dx, dy, dw, dh) = r.door;
            let here = index(a, d, phase, fade);
            if (dx..dx + dw).contains(&x) && (dy..dy + dh).contains(&y) {
                let mut keys = vec![which as u8];
                let mut at = here;
                while at != first {
                    let (before, key) = came[at].unwrap();
                    keys.push(key);
                    at = before;
                }
                keys.reverse();
                return Some(keys.into_iter().map(|k| MOVES[k as usize].map(|m| m.0)).collect());
            }
            let next = index(na, nd, nphase, nfade);
            if came[next].is_none() {
                came[next] = Some((here, which as u8));
                queue.push_back((na, nd, nphase, nfade));
            }
        }
    }
    None
}

/// A player who holds one arrow or none for a tenth of a second at a time can
/// cross all three rooms, and doing so through the game wins it.
#[test]
fn a_player_who_looks_ten_times_a_second_crosses_all_three_rooms() {
    let mut game = Guards::new();
    let mut outcome = Outcome::Playing;
    for room in 0..ROOMS.len() {
        assert_eq!(game.room, room);
        let keys = crossing(room).expect("the room can be crossed");
        'look: for key in keys {
            let mut input = Input::new();
            if let Some(key) = key {
                input.apply(Action::Down(key));
            }
            for _ in 0..LOOK {
                outcome = game.update(&input);
                if outcome != Outcome::Playing || game.room != room {
                    break 'look;
                }
            }
        }
        assert_ne!(outcome, Outcome::Lost, "caught in room {}", room + 1);
    }
    assert_eq!(outcome, Outcome::Won);
}
