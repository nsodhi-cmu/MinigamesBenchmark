use std::convert::Infallible;

use super::replay;
use crate::frame::Frame;
use crate::input::{Action, Button};
use crate::session::Session;

fn fresh() -> Session {
    Session::new("circles", 4).expect("it is cataloged")
}

fn played() -> Session {
    let click = Action::Click { button: Button::Left, x: 320.0, y: 240.0 };
    let mut session = fresh();
    session.advance(&[Action::Wait], 30);
    session.lapse(17);
    session.advance(&[click], 0);
    session.advance(&[Action::Move { x: 100.0, y: 300.0 }], 45);
    session.apply(Action::Press { button: Button::Left, x: 90.0, y: 310.0 });
    session.step();
    session
}

fn png(session: &Session) -> Vec<u8> {
    let mut frame = Frame::new();
    session.draw(&mut frame);
    frame.to_png()
}

#[test]
fn a_replay_shows_every_tick_and_the_screen_each_one_left() {
    let session = played();
    let mut screens = Vec::new();
    let again = replay(fresh(), session.record(), session.tick(), |frame| {
        screens.push(frame.to_png());
        Ok::<_, Infallible>(())
    })
    .expect("nothing stops it");
    assert_eq!((again.outcome(), again.tick()), (session.outcome(), session.tick()));
    assert_eq!(screens.len() as u32, session.tick() + 1);
    assert!(screens.last() == Some(&png(&session)), "the replay ends on a different screen");

    let mut thirty = fresh();
    thirty.advance(&[Action::Wait], 30);
    assert!(screens[30] == png(&thirty), "the thirtieth tick is not the screen it left");
}

#[test]
fn what_lands_between_two_ticks_is_kept_for_the_tick_to_come() {
    let session = played();
    let ticks: Vec<u32> = session.record().iter().map(|(tick, _)| *tick).collect();
    assert_eq!(ticks, [0, 47, 47, 92]);
}
