use super::{LENGTH, REELS, SLOT, Slots, Spin};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input, Key};
use crate::playing::LOOK;

/// Every reel carries every shape and never the same one twice running, so
/// whatever the first reel shows, the other two can be stopped to match it.
#[test]
fn every_reel_holds_every_shape_and_never_one_twice_running() {
    for seed in 0..200u64 {
        for reel in &Slots::new(seed).reels {
            for i in 0..LENGTH {
                assert_ne!(reel.strip[i], reel.strip[(i + 1) % LENGTH], "seed {seed} repeats");
            }
        }
    }
}

/// A reel stopped mid-roll goes no farther than the next shape, and one
/// stopped on a shape stays there.
#[test]
fn a_stopped_reel_goes_on_to_the_next_shape_and_no_further() {
    let mut game = Slots::new(3);
    let mut press = Input::new();
    press.apply(Action::Tap(Key::Space));
    game.reels[0].at = SLOT * 5;
    game.update(&press);
    assert_eq!((game.reels[0].at, game.reels[0].spin), (SLOT * 5, Spin::Still));
    game.reels[1].at = SLOT * 5 + 1;
    game.update(&press);
    let idle = Input::new();
    while game.reels[1].spin != Spin::Still {
        game.update(&idle);
    }
    assert_eq!(game.reels[1].at, SLOT * 6);
}

/// A player who looks only ten times a second, and so can press no more often,
/// can still line up the middle row on every seed.
#[test]
fn looking_ten_times_a_second_is_enough_to_line_up_a_row() {
    let (idle, mut press) = (Input::new(), Input::new());
    press.apply(Action::Tap(Key::Space));
    for seed in 0..40u64 {
        let mut game = Slots::new(seed);
        let mut outcome = Outcome::Playing;
        for reel in 0..REELS {
            let wanted = game.reels[0].showing(1);
            let mut ticks = 0;
            loop {
                let lands = game.reels[reel].at.div_ceil(SLOT) + LENGTH as u32 - 1;
                if reel == 0 || game.reels[reel].strip[lands as usize % LENGTH] == wanted {
                    break;
                }
                for _ in 0..LOOK {
                    assert_eq!(game.update(&idle), Outcome::Playing);
                }
                ticks += LOOK;
                assert!(ticks < 600, "seed {seed} never brought the shape around");
            }
            outcome = game.update(&press);
            while game.reels[reel].spin != Spin::Still {
                outcome = game.update(&idle);
            }
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} was lined up and not won");
    }
}
