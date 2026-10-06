use super::{CARDS, TRAITS, hand, read};
use crate::rng::Rng;

/// A hand is only fair if exactly one trait picks out exactly one card. If two
/// traits did, two answers would be defensible; if none did, none would be.
#[test]
fn exactly_one_card_stands_apart_and_it_is_the_answer() {
    let mut rng = Rng::new(5);
    for round in 0..3000 {
        let (cards, answer, apart) = hand(&mut rng);
        let mut singled = Vec::new();
        for which in TRAITS {
            let values: Vec<usize> = cards.iter().map(|card| read(card, which)).collect();
            let odd: Vec<usize> = (0..CARDS)
                .filter(|slot| values.iter().filter(|value| **value == values[*slot]).count() == 1)
                .collect();
            // None on its own, or every one of them on its own, points at nobody.
            assert!(
                odd.len() != 1 || which == apart,
                "round {round}: {which:?} leaves {odd:?} on its own as well"
            );
            if odd.len() == 1 {
                singled.push((which, odd[0]));
            }
        }
        assert_eq!(
            singled.len(),
            1,
            "round {round}: {singled:?} could each be argued, from {cards:?}"
        );
        assert_eq!(singled[0], (apart, answer), "round {round}: the answer is not the odd card");
    }
}

/// A turn that cannot be seen is not a difference anybody could point at.
#[test]
fn a_card_is_never_set_apart_by_a_turn_that_does_not_show() {
    let mut rng = Rng::new(9);
    for _ in 0..3000 {
        let (cards, answer, apart) = hand(&mut rng);
        if apart == super::Trait::Turn {
            assert!(
                cards.iter().all(|card| card.shape.turns_visibly()),
                "a turned {:?} is the same shape it was",
                cards[answer].shape
            );
        }
    }
}

/// Clicking the odd card out in every round finishes the game.
#[test]
fn picking_the_odd_card_out_every_round_finishes_it() {
    use crate::game::{Game, Outcome};
    use crate::input::{Action, Button, Input};

    for seed in 0..8u64 {
        let mut game = super::Quotients::new(seed);
        let mut input = Input::new();
        let mut outcome = Outcome::Playing;
        for round in 0..super::ROUNDS {
            let (x, y) = super::Quotients::corner(game.odd_one);
            input.apply(Action::Move { x: x + super::WIDE * 0.5, y: y + super::TALL * 0.5 });
            let here = input.cursor();
            input.apply(Action::Press { button: Button::Left, x: here.0, y: here.1 });
            outcome = game.update(&input);
            input.settle();
            let here = input.cursor();
            input.apply(Action::Release { button: Button::Left, x: here.0, y: here.1 });
            input.settle();
            assert_ne!(outcome, Outcome::Lost, "seed {seed} round {round} refused the odd card");
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} did not finish");
    }
}
