use std::collections::HashMap;

use super::{ROUNDS, Stones, center, reply};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

/// Every position one move takes `piles` to.
fn moves(piles: &[u8]) -> Vec<Vec<u8>> {
    (0..piles.len())
        .flat_map(|at| {
            (0..piles[at]).map(move |left| {
                let mut out = piles.to_vec();
                out[at] = left;
                out
            })
        })
        .collect()
}

/// Whether the player to move from `piles` can make sure of taking the last
/// stone, worked out by trying every move rather than by any rule.
fn winning(piles: &[u8], known: &mut HashMap<Vec<u8>, bool>) -> bool {
    if let Some(answer) = known.get(piles) {
        return *answer;
    }
    let answer = moves(piles).iter().any(|after| !winning(after, known));
    known.insert(piles.to_vec(), answer);
    answer
}

/// Clicks that take a pile down to `left` stones.
fn take(game: &mut Stones, input: &mut Input, pile: usize, left: u8) -> Outcome {
    let (x, y) = center(game.piles.len(), pile, left as usize);
    input.apply(Action::Click { button: Button::Left, x, y });
    let outcome = game.update(input);
    input.settle();
    outcome
}

#[test]
fn the_opponent_takes_a_winning_move_from_every_position_that_has_one() {
    let mut known = HashMap::new();
    for code in 0..10u32.pow(4) {
        let piles: Vec<u8> = (0..4).map(|place| (code / 10u32.pow(place) % 10) as u8).collect();
        if piles.iter().all(|pile| *pile == 0) {
            continue;
        }
        let after = reply(&piles);
        assert!(moves(&piles).contains(&after), "{piles:?} is answered by no move at all");
        if winning(&piles, &mut known) {
            assert!(!winning(&after, &mut known), "{piles:?} is answered by a losing move");
        }
    }
}

/// At every turn of every way to win a round against this opponent there is
/// exactly one right move, so a round is not won by luck.
#[test]
fn every_turn_of_every_round_has_exactly_one_right_move() {
    let mut known = HashMap::new();
    for (round, start) in ROUNDS.iter().enumerate() {
        let mut turns = vec![start.to_vec()];
        while let Some(piles) = turns.pop() {
            let right: Vec<Vec<u8>> =
                moves(&piles).into_iter().filter(|after| !winning(after, &mut known)).collect();
            assert_eq!(right.len(), 1, "round {round} at {piles:?}");
            if right[0].iter().any(|pile| *pile > 0) {
                turns.push(reply(&right[0]));
            }
        }
    }
}

#[test]
fn clicking_the_right_stone_every_turn_wins_all_three_rounds() {
    let mut known = HashMap::new();
    let mut game = Stones::new();
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    while outcome == Outcome::Playing {
        let piles = game.piles.clone();
        let right = moves(&piles)
            .into_iter()
            .find(|after| !winning(after, &mut known))
            .expect("a winning player always has a right move");
        let pile = (0..piles.len()).find(|at| right[*at] != piles[*at]).expect("a move takes");
        outcome = take(&mut game, &mut input, pile, right[pile]);
        if outcome == Outcome::Playing && game.piles != ROUNDS[game.round] {
            assert_eq!(game.piles, reply(&right), "the opponent did not answer the move");
        }
    }
    assert_eq!(outcome, Outcome::Won);
}

#[test]
fn taking_one_stone_at_a_time_loses_the_first_round() {
    let mut game = Stones::new();
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    while outcome == Outcome::Playing {
        let pile = game.piles.iter().position(|pile| *pile > 0).expect("stones are left");
        let left = game.piles[pile] - 1;
        outcome = take(&mut game, &mut input, pile, left);
        assert_eq!(game.round, 0);
    }
    assert_eq!(outcome, Outcome::Lost);
}
