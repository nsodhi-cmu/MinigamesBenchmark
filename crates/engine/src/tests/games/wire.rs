use super::{CELL, COLUMNS, Grid, ROWS, Wire, beside, center_of, joined, lay};
use crate::frame;
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

/// The cells of the one route from the dot to the ring.
fn way_through((right, below): &(Grid, Grid)) -> Vec<(usize, usize)> {
    let mut route = vec![(0usize, 0usize)];
    let mut seen = [[false; COLUMNS]; ROWS];
    seen[0][0] = true;
    while *route.last().expect("the route starts somewhere") != (COLUMNS - 1, ROWS - 1) {
        let &(column, row) = route.last().expect("the route starts somewhere");
        let step = beside(column, row)
            .into_iter()
            .find(|&(nc, nr)| !seen[nr][nc] && joined(right, below, (column, row), (nc, nr)));
        match step {
            Some(cell) => {
                seen[cell.1][cell.0] = true;
                route.push(cell);
            }
            None => {
                route.pop();
            }
        }
    }
    route
}

/// Every cell can be walked to, so the maze is a maze and not two of them.
#[test]
fn no_part_of_the_maze_is_walled_off_from_the_rest() {
    for seed in 0..8u64 {
        let (right, below) = lay(seed);
        let mut seen = [[false; COLUMNS]; ROWS];
        let mut edge = vec![(0usize, 0usize)];
        seen[0][0] = true;
        let mut count = 1;
        while let Some((column, row)) = edge.pop() {
            for (nc, nr) in beside(column, row) {
                if !seen[nr][nc] && joined(&right, &below, (column, row), (nc, nr)) {
                    seen[nr][nc] = true;
                    count += 1;
                    edge.push((nc, nr));
                }
            }
        }
        assert_eq!(count, COLUMNS * ROWS, "seed {seed} walls part of itself off");
    }
}

/// Leading the dot from one cell center to the next along the route gets it to
/// the ring without ever touching the wire, so the maze can be dragged through.
#[test]
fn leading_the_dot_from_cell_to_cell_gets_it_to_the_ring() {
    for seed in 0..8u64 {
        let mut game = Wire::new(seed);
        let route = way_through(&lay(seed));
        let mut input = Input::new();
        let start = center_of(0, 0);
        input.apply(Action::Press { button: Button::Left, x: start.0, y: start.1 });
        assert_eq!(game.update(&input), Outcome::Playing, "seed {seed} would not be taken hold of");
        input.settle();

        let mut outcome = Outcome::Playing;
        for (column, row) in route.iter().skip(1) {
            let (x, y) = center_of(*column, *row);
            input.apply(Action::Move { x, y });
            for _ in 0..(CELL / 2.0) as u32 {
                outcome = game.update(&input);
                input.settle();
                assert_ne!(outcome, Outcome::Lost, "seed {seed} lost the dot on the way through");
                if outcome == Outcome::Won {
                    break;
                }
            }
            if outcome == Outcome::Won {
                break;
            }
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} did not reach the ring");
    }
}

#[test]
fn letting_go_of_the_button_drops_the_dot_and_ends_the_round() {
    let mut game = Wire::new(3);
    let mut input = Input::new();
    let start = center_of(0, 0);
    input.apply(Action::Press { button: Button::Left, x: start.0, y: start.1 });
    assert_eq!(game.update(&input), Outcome::Playing);
    input.settle();
    input.apply(Action::Release { button: Button::Left, x: start.0, y: start.1 });
    assert_eq!(game.update(&input), Outcome::Lost, "letting go did not end the round");
}

/// A click is not a hold, so a pointer that touches the dot and comes straight
/// off again never picks it up rather than dropping it at once.
#[test]
fn a_click_on_the_dot_does_not_take_hold_of_it() {
    let mut game = Wire::new(3);
    let mut input = Input::new();
    let start = center_of(0, 0);
    input.apply(Action::Click { button: Button::Left, x: start.0, y: start.1 });
    assert_eq!(game.update(&input), Outcome::Playing);
    input.settle();
    input.apply(Action::Move { x: frame::WIDTH as f32 - 20.0, y: frame::HEIGHT as f32 - 20.0 });
    assert_eq!(game.update(&input), Outcome::Playing, "a click dragged the dot away");
}
