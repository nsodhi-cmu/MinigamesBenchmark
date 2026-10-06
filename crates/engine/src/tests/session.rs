use crate::game::{Outcome, TICK_HZ};
use crate::input::Action;
use crate::session::{Pace, Session};

#[test]
fn advancing_lets_exactly_the_game_time_it_was_given_pass() {
    let mut session = Session::new("serpent", 0).expect("it is cataloged");
    session.advance(&[Action::Wait], 1);
    assert_eq!(session.seconds(), 1.0 / TICK_HZ as f32);
    session.advance(&[Action::Wait], 11);
    assert_eq!(session.seconds(), 12.0 / TICK_HZ as f32);
}

/// No game is won by a player who does nothing. Nothing in a game may move on
/// anything but a tick, so how the game time is cut up between advances cannot
/// change what happens.
#[test]
fn every_game_left_alone_is_lost_the_same_way_however_the_time_is_cut_up() {
    for entry in crate::catalog::CATALOG {
        let run = |ticks| {
            let mut session = Session::new(entry.spec.name, 3).expect("it is cataloged");
            while session.outcome() == Outcome::Playing {
                session.advance(&[Action::Wait], ticks);
            }
            let mut frame = crate::frame::Frame::new();
            session.draw(&mut frame);
            (session.outcome(), session.seconds(), frame.to_png())
        };
        let once = run(6);
        assert_eq!(once.0, Outcome::Lost, "{} is won by doing nothing", entry.spec.name);
        for ticks in [1, 3, 12] {
            let other = run(ticks);
            assert_eq!(
                once.0, other.0,
                "{} ends differently at {ticks} an advance",
                entry.spec.name
            );
            assert_eq!(once.1, other.1, "{} takes longer at {ticks} an advance", entry.spec.name);
            assert!(once.2 == other.2, "{} looks different at {ticks} an advance", entry.spec.name);
        }
    }
}

#[test]
fn nothing_happens_after_an_episode_is_over() {
    let mut session = Session::new("circles", 5).expect("it is cataloged");
    while session.outcome() == Outcome::Playing {
        session.advance(&[Action::Wait], 6);
    }
    let ended = session.seconds();
    session.advance(&[Action::Wait], 6);
    assert_eq!(session.seconds(), ended, "the clock ran on after the round ended");
}

/// A pointer nothing has put anywhere would sit over the middle of the screen,
/// on top of whatever the game draws there.
#[test]
fn the_pointer_is_not_drawn_until_something_has_put_it_somewhere() {
    let picture = |actions: &[Action]| {
        let mut session = Session::new("quotients", 0).expect("it is cataloged");
        session.advance(actions, 1);
        let mut frame = crate::frame::Frame::new();
        session.draw(&mut frame);
        frame.to_png()
    };
    let untouched = picture(&[Action::Wait]);
    let (x, y) = crate::input::Input::new().cursor();
    assert!(
        untouched != picture(&[Action::Move { x, y }]),
        "the pointer was drawn before it was put anywhere"
    );
}

#[test]
fn a_task_turns_the_game_limit_into_the_agent_time_it_takes_at_the_pace() {
    let spec = &crate::catalog::find("maze").expect("it is cataloged").spec;
    let slow = super::task(spec, Pace::Clock(6.try_into().unwrap()));
    assert!(slow.contains("10 times slower") && slow.contains("30 minutes of yours"), "{slow}");
    assert!(slow.ends_with(&spec.prompt()), "the game's own prompt is not the end of the task");
    assert!(super::task(spec, Pace::Clock(45.try_into().unwrap())).contains("1.33 times slower"));
    assert!(super::task(spec, Pace::Clock(TICK_HZ.try_into().unwrap())).contains("normal speed"));
    let stepped = super::task(spec, Pace::Ticks(3.try_into().unwrap()));
    assert!(stepped.contains("lets 3 frames pass") && !stepped.contains("slower"), "{stepped}");
    assert!(stepped.contains("3600 presses of F1"), "{stepped}");
}
