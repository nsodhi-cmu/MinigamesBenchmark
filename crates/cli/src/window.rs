//! A window, a keyboard, and a mouse, handed to the shell.

use minifb::{Key as WinKey, KeyRepeat, MouseButton, MouseMode, Scale, Window, WindowOptions};
use minigame::frame::{Frame, HEIGHT, WIDTH};
use minigame::input::{Action, Button, Key};
use minigame::shell::Shell;
use std::time::Instant;

const KEYS: &[(Key, WinKey)] = &[
    (Key::A, WinKey::A),
    (Key::B, WinKey::B),
    (Key::C, WinKey::C),
    (Key::D, WinKey::D),
    (Key::E, WinKey::E),
    (Key::F, WinKey::F),
    (Key::G, WinKey::G),
    (Key::H, WinKey::H),
    (Key::I, WinKey::I),
    (Key::J, WinKey::J),
    (Key::K, WinKey::K),
    (Key::L, WinKey::L),
    (Key::M, WinKey::M),
    (Key::N, WinKey::N),
    (Key::O, WinKey::O),
    (Key::P, WinKey::P),
    (Key::Q, WinKey::Q),
    (Key::R, WinKey::R),
    (Key::S, WinKey::S),
    (Key::T, WinKey::T),
    (Key::U, WinKey::U),
    (Key::V, WinKey::V),
    (Key::W, WinKey::W),
    (Key::X, WinKey::X),
    (Key::Y, WinKey::Y),
    (Key::Z, WinKey::Z),
    (Key::Num0, WinKey::Key0),
    (Key::Num1, WinKey::Key1),
    (Key::Num2, WinKey::Key2),
    (Key::Num3, WinKey::Key3),
    (Key::Num4, WinKey::Key4),
    (Key::Num5, WinKey::Key5),
    (Key::Num6, WinKey::Key6),
    (Key::Num7, WinKey::Key7),
    (Key::Num8, WinKey::Key8),
    (Key::Num9, WinKey::Key9),
    (Key::Space, WinKey::Space),
    (Key::Enter, WinKey::Enter),
    (Key::Backspace, WinKey::Backspace),
    (Key::Escape, WinKey::Escape),
    (Key::Up, WinKey::Up),
    (Key::Down, WinKey::Down),
    (Key::Left, WinKey::Left),
    (Key::Right, WinKey::Right),
    (Key::Shift, WinKey::LeftShift),
];

const BUTTONS: &[(Button, MouseButton)] =
    &[(Button::Left, MouseButton::Left), (Button::Right, MouseButton::Right)];

/// Naming a game opens it; naming none opens the menu.
pub fn play(game: Option<&str>, seed: u64, rate: f32, showing: bool) -> Result<(), String> {
    let mut shell = Shell::new(game, seed, rate, showing)
        .ok_or_else(|| crate::unknown(game.unwrap_or_default()))?;
    let mut title = shell.title();
    let mut window = Window::new(
        &title,
        WIDTH as usize,
        HEIGHT as usize,
        WindowOptions { scale: Scale::X2, ..WindowOptions::default() },
    )
    .map_err(|e| format!("could not open a window: {e}"))?;
    window.set_target_fps(120);

    let mut frame = Frame::new();
    let mut held_buttons = [false; 2];
    let mut clock = Instant::now();

    while window.is_open() && shell.open() {
        feed(&window, &mut shell, &mut held_buttons);
        let elapsed = clock.elapsed().as_secs_f32();
        clock = Instant::now();
        if let Some(report) = shell.advance(elapsed) {
            let told =
                if shell.spec().varies { format!(" seed {}", shell.seed()) } else { String::new() };
            println!("{} {} {:.2}s{told}", report.game, report.outcome.name(), report.seconds);
        }
        shell.draw(&mut frame);
        if shell.title() != title {
            title = shell.title();
            window.set_title(&title);
        }
        window
            .update_with_buffer(frame.pixels(), WIDTH as usize, HEIGHT as usize)
            .map_err(|e| format!("could not draw: {e}"))?;
    }

    Ok(())
}

/// Every key that went down or came up since the last frame, with repeats,
/// and where the mouse is and what it holds.
fn feed(window: &Window, shell: &mut Shell, held_buttons: &mut [bool; 2]) {
    for &(key, win) in KEYS {
        if window.is_key_pressed(win, KeyRepeat::Yes) {
            shell.act(Action::Down(key));
        }
        if window.is_key_released(win) {
            shell.act(Action::Up(key));
        }
    }
    let Some((x, y)) = window.get_mouse_pos(MouseMode::Clamp) else {
        return;
    };
    shell.act(Action::Move { x, y });
    for &(button, win) in BUTTONS {
        let down = window.get_mouse_down(win);
        if down != held_buttons[button as usize] {
            shell.act(if down {
                Action::Press { button, x, y }
            } else {
                Action::Release { button, x, y }
            });
            held_buttons[button as usize] = down;
        }
    }
}
