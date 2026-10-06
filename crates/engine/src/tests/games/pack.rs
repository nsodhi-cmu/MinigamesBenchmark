use super::{BOARD, BOARDS, Pack, SIDE, SMALL, Shape, TURNS, parts, tray_corner, turned};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

/// Every set of open squares a piece can cover, turned any way but not flipped.
fn placements(shape: &Shape, filled: u64) -> Vec<u64> {
    let mut turns = vec![shape.clone()];
    for _ in 0..3 {
        turns.push(turned(&turns[turns.len() - 1]));
    }
    let mut out = Vec::new();
    for turn in turns {
        for row in 0..SIDE as i32 {
            for column in 0..SIDE as i32 {
                let squares: Option<u64> = turn.iter().try_fold(0, |mask, (r, c)| {
                    let (r, c) = (row + r, column + c);
                    (r < SIDE as i32 && c < SIDE as i32)
                        .then(|| mask | 1u64 << (r as usize * SIDE + c as usize))
                });
                if let Some(mask) = squares.filter(|mask| mask & filled == 0)
                    && !out.contains(&mask)
                {
                    out.push(mask);
                }
            }
        }
    }
    out
}

/// How many ways the pieces fill the grid, stopping at two.
fn packings(fits: &[Vec<u64>], filled: u64, used: usize) -> usize {
    let whole = (1u64 << (SIDE * SIDE)) - 1;
    if filled == whole {
        return 1;
    }
    let first = (!filled).trailing_zeros();
    let mut found = 0;
    for (piece, masks) in fits.iter().enumerate().filter(|(piece, _)| used >> piece & 1 == 0) {
        for mask in masks.iter().filter(|mask| *mask >> first & 1 == 1 && *mask & filled == 0) {
            found += packings(fits, filled | mask, used | 1 << piece);
            if found >= 2 {
                return found;
            }
        }
    }
    found
}

#[test]
fn every_grid_packs_exactly_one_way() {
    for (board, turns) in TURNS.iter().enumerate() {
        let (filled, shapes) = parts(board);
        let fits: Vec<Vec<u64>> = shapes.iter().map(|shape| placements(shape, filled)).collect();
        assert_eq!(packings(&fits, filled, 0), 1, "grid {} does not pack one way", board + 1);
        assert_eq!(turns.len(), shapes.len());
    }
}

/// No two pieces on a grid are the same shape, even with one turned over.
#[test]
fn every_piece_on_a_grid_is_a_different_shape() {
    for board in 0..BOARDS.len() {
        let (_, shapes) = parts(board);
        let canonical = |shape: &Shape| {
            let flipped: Shape = shape.iter().map(|(row, column)| (*row, -*column)).collect();
            let mut forms = Vec::new();
            for start in [shape.clone(), turned(&flipped)] {
                let mut turn = start;
                for _ in 0..4 {
                    let mut sorted = turn.clone();
                    sorted.sort_unstable();
                    forms.push(sorted);
                    turn = turned(&turn);
                }
            }
            forms.into_iter().min().unwrap()
        };
        let mut seen: Vec<Shape> = shapes.iter().map(canonical).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), shapes.len(), "grid {} repeats a piece", board + 1);
    }
}

fn step(game: &mut Pack, input: &mut Input, action: Action) -> Outcome {
    input.apply(action);
    let outcome = game.update(input);
    input.settle();
    outcome
}

fn in_tray(game: &Pack, piece: usize) -> (f32, f32) {
    let shape = &game.pieces[piece].shape;
    let (x, y) = tray_corner(piece, shape);
    (x + (shape[0].1 as f32 + 0.5) * SMALL, y + (shape[0].0 as f32 + 0.5) * SMALL)
}

fn on_grid(square: (i32, i32)) -> (f32, f32) {
    BOARD.middle(square.1 as usize, square.0 as usize)
}

/// Where each piece's first square lies in the answer.
fn answer(board: usize) -> Vec<(i32, i32)> {
    let letters: Vec<char> = BOARDS[board].concat().chars().collect();
    let mut names: Vec<char> = letters.iter().copied().filter(|c| *c != '#').collect();
    names.sort_unstable();
    names.dedup();
    let (_, shapes) = parts(board);
    names
        .iter()
        .zip(shapes)
        .map(|(name, shape)| {
            let cells: Vec<usize> = (0..SIDE * SIDE).filter(|c| letters[*c] == *name).collect();
            let top = cells.iter().map(|c| c / SIDE).min().unwrap() as i32;
            let left = cells.iter().map(|c| c % SIDE).min().unwrap() as i32;
            (top + shape[0].0, left + shape[0].1)
        })
        .collect()
}

/// Turning each piece back beside the grid and dragging it by its first square
/// onto that square's place in the answer fills all three grids.
#[test]
fn turning_each_piece_and_dragging_it_into_place_wins() {
    let mut game = Pack::new();
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    for (board, turns) in TURNS.iter().enumerate() {
        assert_eq!(game.board, board);
        for (piece, square) in answer(board).into_iter().enumerate() {
            assert_eq!(outcome, Outcome::Playing);
            for _ in turns[piece]..4 {
                let (x, y) = in_tray(&game, piece);
                step(&mut game, &mut input, Action::Click { button: Button::Right, x, y });
            }
            let (x, y) = in_tray(&game, piece);
            step(&mut game, &mut input, Action::Press { button: Button::Left, x, y });
            let (x, y) = on_grid(square);
            step(&mut game, &mut input, Action::Move { x, y });
            outcome = step(&mut game, &mut input, Action::Release { button: Button::Left, x, y });
        }
    }
    assert_eq!(outcome, Outcome::Won);
}

/// A piece let go over a square filled from the start is back beside the grid.
#[test]
fn a_piece_let_go_where_it_does_not_fit_goes_back_beside_the_grid() {
    let mut game = Pack::new();
    let mut input = Input::new();
    let (filled, _) = parts(0);
    let blocked = filled.trailing_zeros() as i32;
    let (x, y) = in_tray(&game, 0);
    step(&mut game, &mut input, Action::Press { button: Button::Left, x, y });
    let (x, y) = on_grid((blocked / SIDE as i32, blocked % SIDE as i32));
    step(&mut game, &mut input, Action::Move { x, y });
    step(&mut game, &mut input, Action::Release { button: Button::Left, x, y });
    assert!(game.pieces[0].at.is_none());
}
