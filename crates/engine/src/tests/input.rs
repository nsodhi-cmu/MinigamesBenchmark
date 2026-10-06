use crate::input::{Action, ActionError, Button, Input, Key};

const LEFT: Button = Button::Left;

#[test]
fn every_action_survives_being_written_and_read_back() {
    let actions = [
        Action::Move { x: 12.0, y: 340.0 },
        Action::Press { button: LEFT, x: 5.0, y: 6.0 },
        Action::Release { button: Button::Right, x: 7.0, y: 8.0 },
        Action::Click { button: Button::Right, x: 9.0, y: 10.0 },
        Action::Down(Key::Space),
        Action::Up(Key::Num7),
        Action::Tap(Key::Escape),
        Action::Wait,
    ];
    for action in actions {
        assert_eq!(Action::parse(&action.to_string()), Ok(action));
    }
}

#[test]
fn a_click_is_the_left_button_unless_the_right_is_named() {
    assert_eq!(Action::parse("click 3 4"), Ok(Action::Click { button: LEFT, x: 3.0, y: 4.0 }));
    assert_eq!(
        Action::parse("click right 3 4"),
        Ok(Action::Click { button: Button::Right, x: 3.0, y: 4.0 })
    );
}

#[test]
fn a_button_that_names_no_place_is_refused() {
    for line in ["click", "press left", "release right"] {
        assert!(matches!(Action::parse(line), Err(ActionError::BadPlace { .. })), "{line}");
    }
}

#[test]
fn a_line_with_words_left_over_is_not_an_action() {
    for line in ["click left 320 240 now", "move 320 240 click", "tap UP DOWN", "wait 5"] {
        assert!(matches!(Action::parse(line), Err(ActionError::Leftover(_))), "{line}");
    }
}

#[test]
fn a_placed_button_moves_the_pointer_before_it_acts() {
    let mut input = Input::new();
    input.apply(Action::Press { button: LEFT, x: 40.0, y: 50.0 });
    assert_eq!(input.cursor(), (40.0, 50.0));
    assert!(input.button_held(LEFT));
}

#[test]
fn a_key_that_does_not_exist_is_refused_rather_than_ignored() {
    assert_eq!(Action::parse("down f13"), Err(ActionError::UnknownKey("f13".into())));
}

#[test]
fn a_tap_is_over_by_the_time_the_tick_settles() {
    let mut input = Input::new();
    input.apply(Action::Tap(Key::A));
    assert!(input.pressed(Key::A) && input.released(Key::A) && !input.held(Key::A));
    input.settle();
    assert!(!input.pressed(Key::A) && !input.held(Key::A));
}

#[test]
fn a_held_key_stays_held_across_ticks_but_its_edge_does_not() {
    let mut input = Input::new();
    input.apply(Action::Down(Key::Space));
    input.settle();
    assert!(input.held(Key::Space));
    assert!(!input.pressed(Key::Space));
    input.apply(Action::Down(Key::Space));
    assert!(!input.pressed(Key::Space), "holding a key again is not a fresh press");
}

#[test]
fn the_pointer_cannot_be_put_outside_the_picture() {
    let mut input = Input::new();
    input.apply(Action::Move { x: -400.0, y: 9000.0 });
    let (x, y) = input.cursor();
    assert!(x >= 0.0 && x < crate::frame::WIDTH as f32);
    assert!(y >= 0.0 && y < crate::frame::HEIGHT as f32);
}

#[test]
fn two_arrows_at_once_walk_no_faster_than_one() {
    let mut input = Input::new();
    input.apply(Action::Down(Key::Right));
    let (across, down) = input.heading().expect("an arrow is held");
    assert_eq!((across, down), (1.0, 0.0));
    input.apply(Action::Down(Key::Down));
    let (across, down) = input.heading().expect("two arrows are held");
    assert!((libm::hypotf(across, down) - 1.0).abs() < 1e-6, "a diagonal is longer than a step");
}

#[test]
fn opposite_arrows_held_together_cancel_each_other_out() {
    let mut input = Input::new();
    input.apply(Action::Down(Key::Left));
    input.apply(Action::Down(Key::Right));
    assert_eq!(input.axis(Key::Left, Key::Right), 0.0);
    assert_eq!(input.arrows(), (0.0, 0.0));
    assert_eq!(input.heading(), None, "nothing is walked when the arrows cancel");
}

#[test]
fn a_place_that_is_not_a_number_on_the_screen_is_refused() {
    for line in ["click nan 5", "move inf inf", "press -inf 0", "click 10 infinity"] {
        assert!(Action::parse(line).is_err(), "`{line}` was read as a place");
    }
}
