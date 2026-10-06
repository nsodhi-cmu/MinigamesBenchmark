use super::{CELL, COLUMNS, Maze4, ROWS, goal, spread};
use crate::game::{Game, LIMIT, Outcome};
use crate::input::{Action, Input, Key};
use crate::playing::LOOK;

/// The cells from the start to the exit, the short way around.
fn route(walls: &crate::maze::Walls<COLUMNS, ROWS>) -> Vec<(usize, usize)> {
    let far = spread(walls);
    let mut at = goal(walls);
    let mut back = vec![at];
    while far[at.1 * COLUMNS + at.0] > 0 {
        let step = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)]
            .iter()
            .map(|(dx, dy)| ((at.0 as i32 + dx) as usize, (at.1 as i32 + dy) as usize))
            .find(|next| far[next.1 * COLUMNS + next.0] == far[at.1 * COLUMNS + at.0] - 1)
            .expect("every cell on the way is one step nearer");
        at = step;
        back.push(at);
    }
    back.reverse();
    back
}

#[test]
fn a_player_who_knows_the_way_walks_it_within_the_limit() {
    for seed in 0..8u64 {
        let mut game = Maze4::new(seed);
        let route = route(&game.walls);
        let mut looks = 0;
        let mut outcome = Outcome::Playing;
        while outcome == Outcome::Playing {
            let here = ((game.at.0 / CELL) as usize, (game.at.1 / CELL) as usize);
            let along = route.iter().rposition(|cell| *cell == here).unwrap_or(0);
            let ahead = route[(along + 1).min(route.len() - 1)];
            let (x, y) = game.walls.board.middle(ahead.0, ahead.1);
            let (dx, dy) = (x - game.at.0, y - game.at.1);
            let mut input = Input::new();
            if dx.abs() > dy.abs() {
                input.apply(Action::Down(if dx > 0.0 { Key::Right } else { Key::Left }));
            } else {
                input.apply(Action::Down(if dy > 0.0 { Key::Down } else { Key::Up }));
            }
            for _ in 0..LOOK {
                outcome = game.update(&input);
                if outcome != Outcome::Playing {
                    break;
                }
            }
            looks += 1;
            assert!(looks < LIMIT as u32 * 10, "seed {seed} was still walking at the limit");
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed}");
    }
}
