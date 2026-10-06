use super::{CELLS, Lights, PRESSES, flips, middle};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

/// The squares that clear `white`, if any do, found by elimination over the
/// clicks, where clicking twice is not clicking at all. Each row is a square's
/// flip mask with the clicks that make it up carried beside it.
fn answer(white: u64) -> Option<u64> {
    let mut rows: Vec<(u64, u64)> = (0..CELLS).map(|cell| (flips(cell), 1u64 << cell)).collect();
    let mut target = (white, 0u64);
    let mut pivots = Vec::new();
    for bit in 0..CELLS {
        let Some(at) = (pivots.len()..rows.len()).find(|r| rows[*r].0 >> bit & 1 == 1) else {
            continue;
        };
        rows.swap(pivots.len(), at);
        let pivot = rows[pivots.len()];
        for (r, row) in rows.iter_mut().enumerate() {
            if r != pivots.len() && row.0 >> bit & 1 == 1 {
                *row = (row.0 ^ pivot.0, row.1 ^ pivot.1);
            }
        }
        if target.0 >> bit & 1 == 1 {
            target = (target.0 ^ pivot.0, target.1 ^ pivot.1);
        }
        pivots.push(bit);
    }
    assert_eq!(pivots.len(), CELLS, "some pattern of squares can be clicked two ways");
    (target.0 == 0).then_some(target.1)
}

/// The thirty-six clicks are independent, so every board has exactly one set of
/// squares that clears it. Clicking each board's one answer clears all three,
/// and each answer is as many clicks as the board was dealt from.
#[test]
fn clicking_each_boards_answer_wins_in_as_many_clicks_as_it_was_dealt() {
    for seed in 0..20 {
        let mut game = Lights::new(seed);
        let mut input = Input::new();
        let mut outcome = Outcome::Playing;
        for (board, presses) in PRESSES.iter().enumerate() {
            assert_eq!(game.board, board);
            let clicks = answer(game.white).expect("a dealt board clears");
            assert_eq!(clicks.count_ones() as usize, *presses, "seed {seed} board {board}");
            for cell in (0..CELLS).filter(|cell| clicks >> cell & 1 == 1) {
                assert_eq!(outcome, Outcome::Playing);
                let (x, y) = middle(cell);
                input.apply(Action::Click { button: Button::Left, x, y });
                outcome = game.update(&input);
                input.settle();
            }
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed}");
    }
}
