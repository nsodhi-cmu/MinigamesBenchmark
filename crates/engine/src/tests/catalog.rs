use crate::catalog::CATALOG;
use crate::frame::Frame;
use crate::game::{LIMIT, Outcome};
use crate::input::{Action, Button, KEYS};
use crate::playing::LOOK;
use crate::rng::Rng;
use crate::session::Session;

/// An action stream that presses everything a game might listen for.
fn flail(rng: &mut Rng) -> Vec<Action> {
    let key = KEYS[rng.below(KEYS.len())].0;
    let (x, y) = (rng.range(0.0, 640.0), rng.range(0.0, 480.0));
    let button = if rng.chance(0.8) { Button::Left } else { Button::Right };
    let keyed = [Action::Down(key), Action::Up(key), Action::Tap(key)];
    let pointed = [
        Action::Press { button, x, y },
        Action::Release { button, x, y },
        Action::Click { button, x, y },
    ];
    vec![Action::Move { x, y }, keyed[rng.below(3)], pointed[rng.below(3)]]
}

#[test]
fn every_game_survives_being_played_badly_for_its_whole_limit() {
    for entry in CATALOG {
        for seed in 0..3u64 {
            let mut session = Session::new(entry.spec.name, seed).expect("it is cataloged");
            let mut rng = Rng::new(seed ^ 0xA11);
            let mut frame = Frame::new();
            while session.outcome() == Outcome::Playing {
                session.advance(&flail(&mut rng), LOOK);
                session.draw(&mut frame);
            }
        }
    }
}

/// A game that says the seed decides nothing must really be the same game
/// every time, so that hiding the seed hides nothing a player would want.
#[test]
fn a_game_that_shows_no_seed_is_the_same_game_whatever_the_seed() {
    for entry in CATALOG.iter().filter(|entry| !entry.spec.varies) {
        let shot = |seed: u64| {
            let mut session = Session::new(entry.spec.name, seed).expect("it is cataloged");
            let mut frame = Frame::new();
            let mut rng = Rng::new(7);
            for _ in 0..40 {
                session.advance(&flail(&mut rng), LOOK);
            }
            session.draw(&mut frame);
            frame.to_png()
        };
        assert!(shot(0) == shot(99), "{} hides a seed that changes it", entry.spec.name);
    }
}

#[test]
fn a_seed_and_a_script_decide_the_picture_exactly() {
    for entry in CATALOG {
        let run = || {
            let mut session = Session::new(entry.spec.name, 42).expect("it is cataloged");
            let mut rng = Rng::new(9);
            for _ in 0..60 {
                session.advance(&flail(&mut rng), LOOK);
            }
            let mut frame = Frame::new();
            session.draw(&mut frame);
            frame.to_png()
        };
        assert!(run() == run(), "{} did not repeat itself", entry.spec.name);
    }
}

#[test]
fn a_game_left_alone_ends_when_its_limit_does() {
    let mut session = Session::new("maze", 1).expect("it is cataloged");
    while session.outcome() == Outcome::Playing {
        session.advance(&[Action::Wait], LOOK);
    }
    assert_eq!(session.outcome(), Outcome::Lost);
    assert_eq!(session.seconds(), LIMIT);
}
