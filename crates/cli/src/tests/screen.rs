use super::{Held, key};
use minigame::input::{Action, Button, Key};

#[test]
fn a_letter_is_one_key_whichever_case_the_keyboard_carries_it_in() {
    assert_eq!(key(0x71), Some(Key::Q));
    assert_eq!(key(0x51), Some(Key::Q));
    assert_eq!(key(0x37), Some(Key::Num7));
    assert_eq!(key(0xffe2), Some(Key::Shift));
    assert_eq!(key(0xffe3), None, "control is not a key any game reads");
    assert_eq!(key(0x2c), None, "a comma is not a key any game reads");
}

#[test]
fn between_two_advances_only_what_is_still_held_counts() {
    let mut held = Held::default();
    let landed = Held::default();
    held.apply(Action::Down(Key::Space));
    held.apply(Action::Up(Key::Space));
    held.apply(Action::Press { button: Button::Left, x: 10.0, y: 20.0 });
    held.apply(Action::Release { button: Button::Left, x: 10.0, y: 20.0 });
    assert_eq!(landed.changes(&held), [Action::Move { x: 10.0, y: 20.0 }]);

    held.apply(Action::Down(Key::Right));
    held.apply(Action::Press { button: Button::Left, x: 30.0, y: 40.0 });
    let pressed = held.clone();
    assert_eq!(
        landed.changes(&pressed),
        [
            Action::Move { x: 30.0, y: 40.0 },
            Action::Down(Key::Right),
            Action::Press { button: Button::Left, x: 30.0, y: 40.0 },
        ]
    );
    held.apply(Action::Move { x: 50.0, y: 60.0 });
    held.apply(Action::Release { button: Button::Left, x: 50.0, y: 60.0 });
    assert_eq!(
        pressed.changes(&held),
        [
            Action::Move { x: 50.0, y: 60.0 },
            Action::Release { button: Button::Left, x: 50.0, y: 60.0 }
        ],
        "a key still held is not pressed again, and the drag ends where the pointer went"
    );
}
