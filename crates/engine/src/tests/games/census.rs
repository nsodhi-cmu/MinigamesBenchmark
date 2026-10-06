use crate::game::Outcome;
use crate::input::{Action, Key};
use crate::playing::LOOK;
use crate::session::Session;

/// One press of enter, held down after it as a keyboard would hold it, must
/// give the answer once and leave the round settled.
#[test]
fn one_press_of_enter_answers_once_and_for_all() {
    let mut session = Session::new("census", 0).expect("it is cataloged");
    while session.seconds() < 30.0 {
        session.advance(&[Action::Wait], LOOK);
    }
    session.advance(&[Action::Tap(Key::Num1)], LOOK);
    session.advance(&[Action::Down(Key::Enter)], LOOK);

    let settled = session.outcome();
    let seconds = session.seconds();
    assert_ne!(settled, Outcome::Playing, "the answer was not taken");
    for _ in 0..40 {
        session.advance(&[], LOOK);
    }
    assert_eq!(session.outcome(), settled, "the answer changed after it was given");
    assert_eq!(session.seconds(), seconds, "the clock ran on after the round ended");
}
