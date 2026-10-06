use super::{BOARDS, Paint, square, squares, start};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input, Key};

const STEPS: [(Key, (i32, i32)); 4] =
    [(Key::Left, (-1, 0)), (Key::Right, (1, 0)), (Key::Up, (0, -1)), (Key::Down, (0, 1))];

fn neighbors(board: usize, at: (i32, i32)) -> Vec<(i32, i32)> {
    STEPS
        .iter()
        .map(|(_, (dx, dy))| (at.0 + dx, at.1 + dy))
        .filter(|next| square(board, *next) != b'#')
        .collect()
}

/// Every way to paint the whole board, walking from where the round starts.
/// A way is given up on as soon as some unpainted square can no longer be
/// reached from where the walk stands.
fn ways(board: usize, at: (i32, i32), painted: &mut Vec<(i32, i32)>) -> Vec<Vec<(i32, i32)>> {
    if painted.len() == squares(board).len() {
        return vec![painted.clone()];
    }
    let mut seen = vec![at];
    let mut reach = vec![at];
    while let Some(square) = reach.pop() {
        for next in neighbors(board, square) {
            if !painted.contains(&next) && !seen.contains(&next) {
                seen.push(next);
                reach.push(next);
            }
        }
    }
    if seen.len() != squares(board).len() - painted.len() + 1 {
        return Vec::new();
    }
    let mut found = Vec::new();
    for next in neighbors(board, at) {
        if !painted.contains(&next) {
            painted.push(next);
            found.append(&mut ways(board, next, painted));
            painted.pop();
        }
    }
    found
}

fn walks(board: usize) -> Vec<Vec<(i32, i32)>> {
    let mut painted = vec![start(board)];
    ways(board, start(board), &mut painted)
}

#[test]
fn walking_one_of_the_ways_round_paints_all_three_boards() {
    let mut game = Paint::new();
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    for board in 0..BOARDS.len() {
        assert_eq!(game.board, board);
        let walk = walks(board).pop().expect("a board can be painted");
        for pair in walk.windows(2) {
            assert_eq!(outcome, Outcome::Playing);
            let step = (pair[1].0 - pair[0].0, pair[1].1 - pair[0].1);
            let (key, _) =
                STEPS.iter().find(|(_, way)| *way == step).expect("a step is one square");
            input.apply(Action::Tap(*key));
            outcome = game.update(&input);
            input.settle();
        }
    }
    assert_eq!(outcome, Outcome::Won);
}

/// Walking back the way you came is the one mistake the game ends on, and
/// walking into nothing at all is no mistake.
#[test]
fn stepping_back_onto_paint_ends_the_round_and_walking_into_the_edge_does_nothing() {
    let mut game = Paint::new();
    let mut input = Input::new();
    input.apply(Action::Tap(Key::Up));
    assert_eq!(game.update(&input), Outcome::Playing, "there is nothing above the start");
    input.settle();
    assert_eq!(game.painted.len(), 1);

    input.apply(Action::Tap(Key::Right));
    assert_eq!(game.update(&input), Outcome::Playing);
    input.settle();
    input.apply(Action::Tap(Key::Left));
    assert_eq!(game.update(&input), Outcome::Lost);
}
