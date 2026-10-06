use std::collections::{HashMap, VecDeque};

use super::{BOARD, BOARDS, CELL, Jam, SIDE, Vehicle, parked, reach};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

/// The shortest run of lots that ends with the red car against the gap.
fn shortest(board: usize) -> Vec<Vec<Vehicle>> {
    let start = parked(board);
    let mut came_from: HashMap<Vec<Vehicle>, Vec<Vehicle>> = HashMap::new();
    let mut queue = VecDeque::from([start.clone()]);
    came_from.insert(start.clone(), start.clone());
    while let Some(lot) = queue.pop_front() {
        if lot[0].at + lot[0].length == SIDE {
            let mut path = vec![lot.clone()];
            while path[path.len() - 1] != start {
                path.push(came_from[&path[path.len() - 1]].clone());
            }
            path.reverse();
            return path;
        }
        for which in 0..lot.len() {
            let (low, high) = reach(&lot, which);
            for at in low..=high.min(SIDE - lot[which].length) {
                let mut next = lot.clone();
                next[which].at = at;
                if !came_from.contains_key(&next) {
                    came_from.insert(next.clone(), lot.clone());
                    queue.push_back(next);
                }
            }
        }
    }
    panic!("lot {} cannot be cleared", board + 1);
}

fn drag(game: &mut Jam, v: Vehicle, to: usize) -> Outcome {
    let middle = |along: f32| {
        let (column, row) =
            if v.across { (along, v.lane as f32 + 0.5) } else { (v.lane as f32 + 0.5, along) };
        (BOARD.left + column * CELL, BOARD.top + row * CELL)
    };
    let mut input = Input::new();
    let mut step = |input: &mut Input, action: Action| {
        input.apply(action);
        let outcome = game.update(input);
        input.settle();
        outcome
    };
    let (x, y) = middle(v.at as f32 + 0.5);
    step(&mut input, Action::Move { x, y });
    let here = input.cursor();
    step(&mut input, Action::Press { button: Button::Left, x: here.0, y: here.1 });
    let (x, y) = middle(to as f32 + 0.5);
    step(&mut input, Action::Move { x, y });
    let here = input.cursor();
    step(&mut input, Action::Release { button: Button::Left, x: here.0, y: here.1 })
}

/// Playing each shortest answer by dragging, then pulling the red car out,
/// clears all three lots.
#[test]
fn dragging_each_shortest_answer_and_the_red_car_out_wins() {
    let mut game = Jam::new();
    let mut outcome = Outcome::Playing;
    for board in 0..BOARDS.len() {
        assert_eq!(game.board, board);
        let path = shortest(board);
        for pair in path.windows(2) {
            let which = (0..pair[0].len()).find(|w| pair[0][*w] != pair[1][*w]).unwrap();
            let v = game.vehicles[which];
            drag(&mut game, v, pair[1][which].at);
            assert_eq!(game.vehicles, pair[1]);
        }
        let red = game.vehicles[0];
        outcome = drag(&mut game, red, SIDE);
    }
    assert_eq!(outcome, Outcome::Won);
}

/// A vehicle dragged far past something in its way stops against it.
#[test]
fn nothing_is_dragged_through_another_vehicle() {
    let mut game = Jam::new();
    let before = game.vehicles.clone();
    let red = before[0];
    assert_eq!(drag(&mut game, red, SIDE), Outcome::Playing);
    assert_eq!(game.board, 0);
    assert!(game.vehicles[0].at + red.length < SIDE, "the red car left through a blocked lane");
    assert_eq!(game.vehicles[1..], before[1..]);
}
