use super::{HIDDEN, SIDE, Search, WAYS, deal, line, middle};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};
use crate::rng::Rng;

/// Reads the grid in every place and direction, the long way around, and counts
/// where each hidden word turns up.
#[test]
fn every_hidden_word_can_be_read_in_exactly_one_place() {
    let mut rng = Rng::new(8);
    for _ in 0..200 {
        let (grid, hidden) = deal(&mut rng);
        for h in &hidden {
            let mut seen = 0;
            for row in 0..SIDE as i32 {
                for column in 0..SIDE as i32 {
                    for way in WAYS {
                        let read: String = (0..5)
                            .map(|step| (row + way.0 * step, column + way.1 * step))
                            .take_while(|(r, c)| {
                                (0..SIDE as i32).contains(r) && (0..SIDE as i32).contains(c)
                            })
                            .map(|(r, c)| grid[r as usize * SIDE + c as usize] as char)
                            .collect();
                        seen += usize::from(read == h.word);
                    }
                }
            }
            assert_eq!(seen, 1, "{} can be read {seen} times", h.word);
        }
    }
}

fn drag(game: &mut Search, from: usize, to: usize) -> Outcome {
    let mut input = Input::new();
    let (x, y) = middle(from);
    input.apply(Action::Press { button: Button::Left, x, y });
    game.update(&input);
    input.settle();
    let (x, y) = middle(to);
    input.apply(Action::Move { x, y });
    game.update(&input);
    input.apply(Action::Release { button: Button::Left, x, y });
    game.update(&input)
}

#[test]
fn dragging_from_each_first_letter_to_its_last_finds_them_all() {
    for seed in 0..20 {
        let mut game = Search::new(seed);
        let hidden = game.hidden.clone();
        let mut outcome = Outcome::Playing;
        for h in hidden {
            assert_eq!(outcome, Outcome::Playing);
            let cells = line(h.first, h.way).unwrap();
            outcome = drag(&mut game, cells[0], cells[4]);
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed}");
    }
}

/// A word dragged from its last letter back to its first is not found.
#[test]
fn dragging_a_word_from_its_last_letter_finds_nothing() {
    let mut game = Search::new(3);
    let h = game.hidden[0];
    let cells = line(h.first, h.way).unwrap();
    drag(&mut game, cells[4], cells[0]);
    assert_eq!(game.found, [false; HIDDEN]);
}
