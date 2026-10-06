use super::{Ahead, BOARDS, Isles, ahead, find};
use crate::game::{Game, Outcome};
use crate::input::{ARROWS, Action, Input, Key};

/// Where a slide from `from` ends up, or nothing if it goes into the water.
fn slid(board: usize, from: (i32, i32), step: (i32, i32)) -> Option<(i32, i32)> {
    let mut at = from;
    loop {
        let next = (at.0 + step.0, at.1 + step.1);
        match ahead(board, next) {
            Ahead::Rock => return Some(at),
            Ahead::Water => return None,
            Ahead::Goal => return Some(next),
            Ahead::Ice => at = next,
        }
    }
}

/// Every sequence of slides that reaches the goal without stopping anywhere twice.
fn routes(board: usize, from: (i32, i32), seen: &mut Vec<(i32, i32)>) -> Vec<Vec<Key>> {
    let mut found = Vec::new();
    for (key, step) in ARROWS {
        let Some(at) = slid(board, from, step) else { continue };
        if at == find(board, b'E') {
            found.push(vec![key]);
        } else if at != from && !seen.contains(&at) {
            seen.push(at);
            for mut route in routes(board, at, seen) {
                route.insert(0, key);
                found.push(route);
            }
            seen.pop();
        }
    }
    found
}

/// One slide, played for as long as it takes to stop.
fn slide(game: &mut Isles, key: Key) -> Outcome {
    let mut input = Input::new();
    input.apply(Action::Tap(key));
    let mut outcome = game.update(&input);
    input.settle();
    while outcome == Outcome::Playing && game.slide.is_some() {
        outcome = game.update(&input);
    }
    outcome
}

/// One route, so that the isle is a puzzle.
#[test]
fn every_isle_has_one_way_across() {
    for board in 0..BOARDS.len() {
        let start = find(board, b'S');
        let found = routes(board, start, &mut vec![start]);
        assert_eq!(found.len(), 1, "isle {board} has {} ways across", found.len());
    }
}

#[test]
fn sliding_the_one_way_across_all_three_isles_wins() {
    let mut game = Isles::new();
    for board in 0..BOARDS.len() {
        let start = find(board, b'S');
        let route = routes(board, start, &mut vec![start]).remove(0);
        for key in route {
            let outcome = slide(&mut game, key);
            let last = board + 1 == BOARDS.len() && game.done == BOARDS.len();
            assert_eq!(outcome, if last { Outcome::Won } else { Outcome::Playing });
        }
        assert_eq!(game.done, board + 1);
    }
}

#[test]
fn a_slide_that_reaches_the_edge_goes_into_the_water() {
    let mut game = Isles::new();
    let start = find(0, b'S');
    let (key, _) = ARROWS.into_iter().find(|(_, step)| slid(0, start, *step).is_none()).unwrap();
    assert_eq!(slide(&mut game, key), Outcome::Lost);
    assert!(!(0..8).contains(&game.at.0) || !(0..8).contains(&game.at.1));
}
