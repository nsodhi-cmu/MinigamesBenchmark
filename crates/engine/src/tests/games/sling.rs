use super::{BUCKETS, BUCKETS_SET, MOUTH, Sling, cast_from, lands_in};
use crate::game::{Game, Outcome};
use crate::input::{Action, Button, Input};

/// Every place the pointer could be let go of that throws the ball up and to
/// the right, which is every throw worth making.
fn drags() -> impl Iterator<Item = (f32, f32)> {
    (0..76).flat_map(|ix| (0..72).map(move |iy| (ix as f32 * 2.0, 336.0 + iy as f32 * 2.0)))
}

/// Every bucket can be thrown into, and by a throw with room on either side of
/// it: there is a place the pointer can be let go of where moving it four
/// either way, in any direction, still lands the ball in the bucket.
#[test]
fn every_bucket_can_be_thrown_into_without_the_pointer_being_placed_exactly() {
    for bucket in 0..BUCKETS {
        let lands = |x: f32, y: f32| lands_in(bucket, cast_from(x, y));
        let solid = drags().any(|(x, y)| {
            [-4.0f32, 0.0, 4.0]
                .iter()
                .all(|dx| [-4.0f32, 0.0, 4.0].iter().all(|dy| lands(x + dx, y + dy)))
        });
        assert!(solid, "bucket {} has no throw with room on either side of it", bucket + 1);
    }
}

/// No bucket hangs off the picture.
#[test]
fn every_bucket_stands_inside_the_picture() {
    use crate::frame;
    for (number, (left, lip)) in BUCKETS_SET.iter().enumerate() {
        assert!(*left > 0.0 && left + MOUTH < frame::WIDTH as f32, "bucket {number} runs off");
        assert!(*lip > 40.0, "bucket {number} hangs off the top of the screen");
    }
}

/// Throwing into all five buckets, one after another, finishes the round, and
/// a throw that misses ends it.
#[test]
fn throwing_into_all_five_buckets_finishes_and_one_miss_ends_it() {
    let good = |bucket: usize| {
        drags()
            .map(|(x, y)| cast_from(x, y))
            .find(|throw| lands_in(bucket, *throw))
            .expect("every bucket can be thrown into")
    };
    let mut game = Sling::new();
    let mut outcome = Outcome::Playing;
    for bucket in 0..BUCKETS {
        assert_eq!(game.bucket, bucket, "the round is not on bucket {}", bucket + 1);
        outcome = throw(&mut game, good(bucket));
        assert_ne!(outcome, Outcome::Lost, "a throw that lands was not counted");
    }
    assert_eq!(outcome, Outcome::Won, "five good throws did not finish the round");

    let mut missed = Sling::new();
    assert_eq!(throw(&mut missed, (60.0, -900.0)), Outcome::Lost, "a miss was forgiven");
}

/// Drag the sling back to the speed asked for, let go, and wait it out.
fn throw(game: &mut Sling, pace: (f32, f32)) -> Outcome {
    let mut input = Input::new();
    input.apply(Action::Move { x: super::POST.0, y: super::POST.1 });
    let here = input.cursor();
    input.apply(Action::Press { button: Button::Left, x: here.0, y: here.1 });
    game.update(&input);
    input.settle();
    let back = (super::POST.0 - pace.0 / super::PULL, super::POST.1 - pace.1 / super::PULL);
    input.apply(Action::Move { x: back.0, y: back.1 });
    game.update(&input);
    input.settle();
    let here = input.cursor();
    input.apply(Action::Release { button: Button::Left, x: here.0, y: here.1 });
    let mut outcome = game.update(&input);
    input.settle();
    for _ in 0..60 * 20 {
        outcome = game.update(&input);
        input.settle();
        if outcome != Outcome::Playing || (game.ball.is_none() && game.settling == 0) {
            break;
        }
    }
    outcome
}
