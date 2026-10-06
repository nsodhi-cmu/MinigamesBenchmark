use super::{SIDE, placements, reasons_out, runs};

#[test]
fn a_run_reads_back_the_way_it_was_laid() {
    for mask in [0u8, 0b0001_1111, 0b0001_0101, 0b0000_1110, 0b0001_1001] {
        for laid in placements(&runs(mask)) {
            assert_eq!(runs(laid), runs(mask), "a placement of the same clue reads differently");
        }
        assert!(
            placements(&runs(mask)).contains(&mask),
            "{mask:08b} is not among its own placements"
        );
    }
}

/// What keeps a dealt puzzle fair is that `reasons_out` says no to the unfair
/// ones, and it is only ever asked about puzzles it has already said yes to.
/// These are the two ways it has to be able to say no.
#[test]
fn numbers_that_two_pictures_fit_are_refused() {
    // One square in the first two rows and one in the first two columns: the
    // pair can sit on either diagonal, so the numbers do not say which.
    let mut lines = vec![Vec::new(); SIDE];
    lines[0] = vec![1];
    lines[1] = vec![1];
    assert!(!reasons_out(&lines, &lines), "numbers that two pictures fit were taken as fair");
}

#[test]
fn numbers_that_no_picture_fits_are_refused() {
    // A full first row across, and nothing filled anywhere down.
    let mut down = vec![Vec::new(); SIDE];
    down[0] = vec![SIDE];
    let across = vec![Vec::new(); SIDE];
    assert!(!reasons_out(&down, &across), "numbers that nothing fits were taken as fair");
}

/// Filling exactly the squares the picture calls for finishes the grid.
#[test]
fn filling_what_the_numbers_describe_finishes_the_grid() {
    use crate::game::{Game, Outcome};
    use crate::input::{Action, Button, Input};

    for seed in 0..6u64 {
        let mut game = super::Glyph::new(seed);
        let mut input = Input::new();
        let board = super::Glyph::board();
        let (ox, oy) = (board.left, board.top);
        let mut outcome = Outcome::Playing;
        for row in 0..SIDE {
            for column in 0..SIDE {
                if game.answer[row] >> column & 1 == 0 {
                    continue;
                }
                input.apply(Action::Move {
                    x: ox + column as f32 * super::CELL + super::CELL * 0.5,
                    y: oy + row as f32 * super::CELL + super::CELL * 0.5,
                });
                let here = input.cursor();
                input.apply(Action::Press { button: Button::Left, x: here.0, y: here.1 });
                outcome = game.update(&input);
                input.settle();
                let here = input.cursor();
                input.apply(Action::Release { button: Button::Left, x: here.0, y: here.1 });
                input.settle();
            }
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} was filled in right and did not finish");
    }
}
