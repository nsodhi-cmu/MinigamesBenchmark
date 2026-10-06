//! What a player may do, and what a game sees of it.
//!
//! The vocabulary is the one a computer-use agent already has: put the pointer
//! somewhere, hold or let go of a button, and hold or let go of a key.

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    Left,
    Right,
}

impl Button {
    fn parse(word: &str) -> Option<Button> {
        match word {
            "left" => Some(Button::Left),
            "right" => Some(Button::Right),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Space,
    Enter,
    Backspace,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Shift,
}

pub const KEYS: [(Key, &str); Key::COUNT] = [
    (Key::A, "A"),
    (Key::B, "B"),
    (Key::C, "C"),
    (Key::D, "D"),
    (Key::E, "E"),
    (Key::F, "F"),
    (Key::G, "G"),
    (Key::H, "H"),
    (Key::I, "I"),
    (Key::J, "J"),
    (Key::K, "K"),
    (Key::L, "L"),
    (Key::M, "M"),
    (Key::N, "N"),
    (Key::O, "O"),
    (Key::P, "P"),
    (Key::Q, "Q"),
    (Key::R, "R"),
    (Key::S, "S"),
    (Key::T, "T"),
    (Key::U, "U"),
    (Key::V, "V"),
    (Key::W, "W"),
    (Key::X, "X"),
    (Key::Y, "Y"),
    (Key::Z, "Z"),
    (Key::Num0, "0"),
    (Key::Num1, "1"),
    (Key::Num2, "2"),
    (Key::Num3, "3"),
    (Key::Num4, "4"),
    (Key::Num5, "5"),
    (Key::Num6, "6"),
    (Key::Num7, "7"),
    (Key::Num8, "8"),
    (Key::Num9, "9"),
    (Key::Space, "SPACE"),
    (Key::Enter, "ENTER"),
    (Key::Backspace, "BACKSPACE"),
    (Key::Escape, "ESC"),
    (Key::Up, "UP"),
    (Key::Down, "DOWN"),
    (Key::Left, "LEFT"),
    (Key::Right, "RIGHT"),
    (Key::Shift, "SHIFT"),
];

/// The four arrows and the step each one takes, in the order a game reads them
/// where more than one is pressed on the same tick.
pub const ARROWS: [(Key, (i32, i32)); 4] =
    [(Key::Up, (0, -1)), (Key::Down, (0, 1)), (Key::Left, (-1, 0)), (Key::Right, (1, 0))];

/// The keys a game rises on, for jumping and for thrusting.
pub const LIFT: [Key; 2] = [Key::Up, Key::Space];

impl Key {
    pub const COUNT: usize = Key::Shift as usize + 1;

    pub fn name(self) -> &'static str {
        KEYS[self as usize].1
    }

    pub fn parse(word: &str) -> Option<Key> {
        let upper = word.to_ascii_uppercase();
        KEYS.iter().find(|(_, name)| *name == upper).map(|(key, _)| *key)
    }

    /// The character this key stands for where it stands for one.
    pub fn character(self) -> Option<char> {
        if self == Key::Space {
            return Some(' ');
        }
        let mut chars = self.name().chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Some(c.to_ascii_lowercase()),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Action {
    Move { x: f32, y: f32 },
    Press { button: Button, x: f32, y: f32 },
    Release { button: Button, x: f32, y: f32 },
    Click { button: Button, x: f32, y: f32 },
    Down(Key),
    Up(Key),
    Tap(Key),
    Wait,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionError {
    Empty,
    UnknownVerb(String),
    UnknownKey(String),
    BadPlace { verb: String, got: String },
    Leftover(String),
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::Empty => write!(f, "no action given"),
            ActionError::UnknownVerb(w) => write!(f, "no such action: {w}"),
            ActionError::UnknownKey(w) => write!(f, "no such key: {w}"),
            ActionError::BadPlace { verb, got } => {
                write!(f, "{verb} wants a place, two numbers: {verb} <x> <y>, got `{got}`")
            }
            ActionError::Leftover(w) => {
                write!(f, "`{w}` is left over after the action; send one action to a line")
            }
        }
    }
}

impl std::error::Error for ActionError {}

impl Action {
    /// One line of the action language, as a record holds it.
    ///
    /// ```text
    /// click|press|release [left|right] X Y    move X Y
    /// down|up|tap KEY                         wait
    /// ```
    pub fn parse(line: &str) -> Result<Action, ActionError> {
        let lower = line.trim().to_ascii_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        let (&verb, rest) = words.split_first().ok_or(ActionError::Empty)?;

        let ended = |left: &[&str]| match left {
            [] => Ok(()),
            _ => Err(ActionError::Leftover(left.join(" "))),
        };
        let unplaced = || ActionError::BadPlace { verb: verb.to_string(), got: rest.join(" ") };
        let place = |words: &[&str]| match words {
            // `nan` and `inf` parse as numbers and are not places: a pointer
            // put at one would stay there, since nothing compares true to it.
            [x, y, left @ ..] => match (x.parse::<f32>(), y.parse::<f32>()) {
                (Ok(x), Ok(y)) if x.is_finite() && y.is_finite() => ended(left).map(|()| (x, y)),
                _ => Err(unplaced()),
            },
            _ => Err(unplaced()),
        };
        let pointed = |words: &[&str]| {
            let (button, words) =
                match words.split_first().map(|(w, after)| (Button::parse(w), after)) {
                    Some((Some(button), after)) => (button, after),
                    _ => (Button::Left, words),
                };
            place(words).map(|(x, y)| (button, x, y))
        };
        let key = |words: &[&str]| match words {
            [name, left @ ..] => {
                let key =
                    Key::parse(name).ok_or_else(|| ActionError::UnknownKey(name.to_string()))?;
                ended(left).map(|()| key)
            }
            [] => Err(ActionError::UnknownKey(format!("nothing, and {verb} wants a KEY"))),
        };

        match verb {
            "move" => place(rest).map(|(x, y)| Action::Move { x, y }),
            "press" => pointed(rest).map(|(button, x, y)| Action::Press { button, x, y }),
            "release" => pointed(rest).map(|(button, x, y)| Action::Release { button, x, y }),
            "click" => pointed(rest).map(|(button, x, y)| Action::Click { button, x, y }),
            "down" => key(rest).map(Action::Down),
            "up" => key(rest).map(Action::Up),
            "tap" => key(rest).map(Action::Tap),
            "wait" => ended(rest).map(|()| Action::Wait),
            other => Err(ActionError::UnknownVerb(other.to_string())),
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let placed =
            |f: &mut fmt::Formatter<'_>, verb: &str, button: Button, x: f32, y: f32| match button {
                Button::Left => write!(f, "{verb} {} {}", x.round(), y.round()),
                Button::Right => write!(f, "{verb} right {} {}", x.round(), y.round()),
            };
        match *self {
            Action::Move { x, y } => write!(f, "move {} {}", x.round(), y.round()),
            Action::Press { button, x, y } => placed(f, "press", button, x, y),
            Action::Release { button, x, y } => placed(f, "release", button, x, y),
            Action::Click { button, x, y } => placed(f, "click", button, x, y),
            Action::Down(k) => write!(f, "down {}", k.name()),
            Action::Up(k) => write!(f, "up {}", k.name()),
            Action::Tap(k) => write!(f, "tap {}", k.name()),
            Action::Wait => write!(f, "wait"),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct State {
    held: bool,
    pressed: bool,
    released: bool,
}

/// Everything a game is told about the player, this tick.
pub struct Input {
    cursor: (f32, f32),
    keys: [State; Key::COUNT],
    buttons: [State; 2],
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Input {
            cursor: (crate::frame::WIDTH as f32 * 0.5, crate::frame::HEIGHT as f32 * 0.5),
            keys: [State::default(); Key::COUNT],
            buttons: [State::default(); 2],
        }
    }

    pub fn cursor(&self) -> (f32, f32) {
        self.cursor
    }

    pub fn held(&self, key: Key) -> bool {
        self.keys[key as usize].held
    }

    pub fn pressed(&self, key: Key) -> bool {
        self.keys[key as usize].pressed
    }

    pub fn released(&self, key: Key) -> bool {
        self.keys[key as usize].released
    }

    /// Minus one, zero, or one: which of a pair of keys is held.
    pub fn axis(&self, minus: Key, plus: Key) -> f32 {
        f32::from(self.held(plus)) - f32::from(self.held(minus))
    }

    /// Which way the arrows held point, across and down.
    pub fn arrows(&self) -> (f32, f32) {
        (self.axis(Key::Left, Key::Right), self.axis(Key::Up, Key::Down))
    }

    /// The same as a step of length one, or nothing while no arrow is held, so
    /// that walking two arrows at once is no faster than walking one.
    pub fn heading(&self) -> Option<(f32, f32)> {
        let (across, down) = self.arrows();
        let length = (across * across + down * down).sqrt();
        (length > 0.0).then(|| (across / length, down / length))
    }

    /// The one arrow a tick counts as pressed. Where more than one was, they
    /// are taken in the order they are named, so that two at once always
    /// answer the same way.
    pub fn arrow(&self) -> Option<(Key, (i32, i32))> {
        ARROWS.into_iter().find(|(key, _)| self.pressed(*key))
    }

    /// Whether the key a game rises on is down. UP and SPACE both do it, so
    /// that a player who reaches for either is answered.
    pub fn lift_held(&self) -> bool {
        LIFT.iter().any(|key| self.held(*key))
    }

    /// The same for the tick it goes down, for a game that jumps rather than
    /// thrusts.
    pub fn lift_pressed(&self) -> bool {
        LIFT.iter().any(|key| self.pressed(*key))
    }

    pub fn button_held(&self, button: Button) -> bool {
        self.buttons[button as usize].held
    }

    pub fn button_pressed(&self, button: Button) -> bool {
        self.buttons[button as usize].pressed
    }

    pub fn button_released(&self, button: Button) -> bool {
        self.buttons[button as usize].released
    }

    /// The first key pressed this tick that stands for a character.
    pub fn typed(&self) -> Option<char> {
        KEYS.iter().find(|(k, _)| self.pressed(*k)).and_then(|(k, _)| k.character())
    }

    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Move { x, y } => self.point(x, y),
            Action::Press { button, x, y } => {
                self.point(x, y);
                Self::down(&mut self.buttons[button as usize]);
            }
            Action::Release { button, x, y } => {
                self.point(x, y);
                Self::up(&mut self.buttons[button as usize]);
            }
            Action::Click { button, x, y } => {
                self.point(x, y);
                Self::tap(&mut self.buttons[button as usize]);
            }
            Action::Down(k) => Self::down(&mut self.keys[k as usize]),
            Action::Up(k) => Self::up(&mut self.keys[k as usize]),
            Action::Tap(k) => Self::tap(&mut self.keys[k as usize]),
            Action::Wait => {}
        }
    }

    /// Forget this tick's edges, keeping what is still held down.
    pub fn settle(&mut self) {
        for state in self.keys.iter_mut().chain(self.buttons.iter_mut()) {
            state.pressed = false;
            state.released = false;
        }
    }

    /// Let go of everything, as an episode ending should.
    pub fn clear(&mut self) {
        self.keys = [State::default(); Key::COUNT];
        self.buttons = [State::default(); 2];
    }

    fn point(&mut self, x: f32, y: f32) {
        self.cursor = (
            x.clamp(0.0, crate::frame::WIDTH as f32 - 1.0),
            y.clamp(0.0, crate::frame::HEIGHT as f32 - 1.0),
        );
    }

    fn down(state: &mut State) {
        if !state.held {
            state.pressed = true;
        }
        state.held = true;
    }

    fn up(state: &mut State) {
        if state.held {
            state.released = true;
        }
        state.held = false;
    }

    fn tap(state: &mut State) {
        state.pressed = true;
        state.released = true;
        state.held = false;
    }
}

#[cfg(test)]
#[path = "tests/input.rs"]
mod tests;
