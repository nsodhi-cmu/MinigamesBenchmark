use super::{Block, QUOTA, SLACK, Sword, WAYS, cuts_the_way, way_of};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

#[test]
fn a_stroke_along_the_arrow_cuts_whichever_way_it_points() {
    for step in 0..WAYS {
        let (wx, wy) = way_of(step);
        let from = (320.0, 240.0);
        let along = (from.0 + wx * 120.0, from.1 + wy * 120.0);
        assert!(cuts_the_way(from, along, step), "way {step} refused its own direction");
    }
}

/// One step around the clock is thirty degrees, so the slack has to leave the
/// neighboring ways out or two answers would be right.
#[test]
fn no_stroke_can_answer_two_ways_at_once() {
    assert!(SLACK < std::f32::consts::TAU / WAYS as f32 / 2.0, "the slack overlaps its neighbors");
    for step in 0..WAYS {
        for other in 0..WAYS {
            if other == step {
                continue;
            }
            let (wx, wy) = way_of(other);
            let from = (320.0, 240.0);
            let stroke = (from.0 + wx * 130.0, from.1 + wy * 130.0);
            assert!(
                !cuts_the_way(from, stroke, step),
                "a stroke along way {other} was taken for way {step}"
            );
        }
    }
}

#[test]
fn a_flick_too_short_to_be_meant_is_not_a_cut() {
    let (wx, wy) = way_of(3);
    let from = (300.0, 200.0);
    let nudge = (from.0 + wx * 12.0, from.1 + wy * 12.0);
    assert!(!cuts_the_way(from, nudge, 3), "a twelve pixel nudge counted as a cut");
}

/// Cutting each block the way its arrow points, ten times over, finishes it.
#[test]
fn cutting_the_way_the_arrow_points_finishes_the_round() {
    for seed in 0..6u64 {
        let mut game = Sword::new(seed);
        let mut input = Input::new();
        let mut outcome = Outcome::Playing;
        for block in 0..QUOTA {
            while !matches!(game.block, Block::Resting { .. }) {
                game.update(&input);
                input.settle();
            }
            let (at, step) = match game.block {
                Block::Resting { step } => (super::REST, step),
                _ => panic!("block {block} never came to rest"),
            };
            let (wx, wy) = way_of(step);
            let middle = crate::frame::WIDTH as f32 * 0.5;
            input.apply(Action::Move { x: middle - wx * 110.0, y: at - wy * 110.0 });
            let here = input.cursor();
            input.apply(Action::Press { button: Button::Left, x: here.0, y: here.1 });
            game.update(&input);
            input.settle();
            input.apply(Action::Move { x: middle + wx * 110.0, y: at + wy * 110.0 });
            let here = input.cursor();
            input.apply(Action::Release { button: Button::Left, x: here.0, y: here.1 });
            outcome = game.update(&input);
            input.settle();
            assert_ne!(outcome, Outcome::Lost, "seed {seed} refused a cut along its own arrow");

            while matches!(game.block, Block::Split { .. }) && outcome == Outcome::Playing {
                outcome = game.update(&input);
                input.settle();
            }
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} did not finish");
    }
}

/// Cutting one the wrong way ends it, which is what the arrow is for.
#[test]
fn cutting_against_the_arrow_ends_the_round() {
    let mut game = Sword::new(3);
    let mut input = Input::new();
    while !matches!(game.block, Block::Resting { .. }) {
        game.update(&input);
        input.settle();
    }
    let (at, step) = match game.block {
        Block::Resting { step } => (super::REST, step),
        _ => panic!("nothing to cut"),
    };
    let (wx, wy) = way_of(step);
    let middle = crate::frame::WIDTH as f32 * 0.5;
    input.apply(Action::Move { x: middle + wx * 110.0, y: at + wy * 110.0 });
    let here = input.cursor();
    input.apply(Action::Press { button: Button::Left, x: here.0, y: here.1 });
    game.update(&input);
    input.settle();
    input.apply(Action::Move { x: middle - wx * 110.0, y: at - wy * 110.0 });
    let here = input.cursor();
    input.apply(Action::Release { button: Button::Left, x: here.0, y: here.1 });
    assert_eq!(game.update(&input), Outcome::Lost, "a backwards cut was accepted");
}

/// A click is a press and a release on one tick, which makes a stroke of no
/// length. That is not a cut, so it is let go of rather than answered.
#[test]
fn one_click_on_the_block_is_not_a_cut_the_wrong_way() {
    let mut sword = Sword::new(0);
    let idle = Input::new();
    for _ in 0..600 {
        assert_eq!(sword.update(&idle), Outcome::Playing, "the round ended on its own");
        if matches!(sword.block, Block::Resting { .. }) {
            break;
        }
    }
    assert!(matches!(sword.block, Block::Resting { .. }), "the block never came to rest");
    let mut input = Input::new();
    input.apply(Action::Click { button: Button::Left, x: 320.0, y: 300.0 });
    assert_eq!(sword.update(&input), Outcome::Playing, "one click ended the round");
}
