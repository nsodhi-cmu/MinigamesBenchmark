//! One round of one game on a virtual X display of its own, which an agent beside it sees and
//! works with the mouse and the keyboard.

use minigame::frame::{Frame, HEIGHT, WIDTH};
use minigame::game::Outcome;
use minigame::input::{Action, Button, Key};
use minigame::session::{Pace, Session};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};
use x11rb::connection::{Connection, RequestConnection};
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;
use x11rb::{COPY_DEPTH_FROM_PARENT, COPY_FROM_PARENT, CURRENT_TIME};

/// The display the game draws on, and the one an agent beside it is pointed at.
const DISPLAY: &str = ":0";

/// The keysym of F1, `session::ADVANCE`.
const ADVANCE: Keysym = 0xffbe;

/// What the display and standard input tell the round.
enum Heard {
    /// An action, with the moment it was read.
    Landed(Instant, Action),
    /// The advance key went down.
    Advance,
    /// Standard input said to stop, or closed.
    Stop,
    /// The display stopped answering or refused a request.
    Gone,
}

/// Plays `game` on `seed` at `pace`. Standard output says `ready` once the first frame is on
/// the screen, and after `go` on standard input the round runs until it is over or standard
/// input stops it, saying `advanced` after each press of the advance key when paced in ticks.
/// Then every action that landed is printed as its tick and a line of the action language, and
/// last `over`, the outcome, and the tick it ended on.
pub fn play(game: &str, seed: u64, pace: Pace) -> Result<(), String> {
    let mut session = Session::new(game, seed).ok_or_else(|| crate::unknown(game))?;
    let _server = Server::start()?;
    let screen = Screen::open()?;
    let mut frame = Frame::new();
    session.draw(&mut frame);
    screen.draw(&frame)?;

    let (sender, heard) = mpsc::channel();
    let listened = Arc::clone(&screen.connection);
    let listening = sender.clone();
    std::thread::spawn(move || {
        listen(&listened, &listening);
        let _ = listening.send(Heard::Gone);
    });
    screen.connection.get_input_focus().map_err(broken)?.reply().map_err(broken)?;

    println!("ready");
    std::io::stdout().flush().map_err(broken)?;
    let mut told = String::new();
    std::io::stdin().lock().read_line(&mut told).map_err(broken)?;
    if told.trim() != "go" {
        return Err(format!("told `{}` where `go` was wanted", told.trim()));
    }
    std::thread::spawn(move || {
        let _ = std::io::stdin().lock().read_line(&mut String::new());
        let _ = sender.send(Heard::Stop);
    });

    match pace {
        Pace::Clock(rate) => clocked(&mut session, &screen, &heard, rate.get())?,
        Pace::Ticks(ticks) => stepped(&mut session, &screen, &heard, ticks.get())?,
    }

    let mut out = std::io::stdout().lock();
    for (tick, action) in session.record() {
        writeln!(out, "{tick} {action}").map_err(broken)?;
    }
    writeln!(out, "over {} {}", session.outcome().name(), session.tick()).map_err(broken)?;
    out.flush().map_err(broken)
}

/// Ticks due every `1 / rate` seconds after now, each action landing on the first tick not yet
/// due when it was read. Late ticks are played late, and of a run of them only the last is drawn.
fn clocked(
    session: &mut Session,
    screen: &Screen,
    heard: &Receiver<Heard>,
    rate: u32,
) -> Result<(), String> {
    let start = Instant::now();
    let due = |tick: u32| start + Duration::from_secs_f64(f64::from(tick) / f64::from(rate));
    let mut frame = Frame::new();
    let mut drawn = 0;
    while session.outcome() == Outcome::Playing {
        let next = due(session.tick() + 1);
        match heard.recv_timeout(next.saturating_duration_since(Instant::now())) {
            Ok(Heard::Landed(arrived, action)) => {
                while session.outcome() == Outcome::Playing && due(session.tick() + 1) <= arrived {
                    session.step();
                }
                session.apply(action);
            }
            Ok(Heard::Advance) => {}
            Ok(Heard::Stop) => return Ok(()),
            Err(RecvTimeoutError::Timeout) => session.step(),
            Ok(Heard::Gone) | Err(RecvTimeoutError::Disconnected) => return Err(GONE.into()),
        }
        let late = due(session.tick() + 1) <= Instant::now();
        if drawn != session.tick() && (session.outcome() != Outcome::Playing || !late) {
            session.draw(&mut frame);
            screen.draw(&frame)?;
            drawn = session.tick();
        }
    }
    Ok(())
}

/// The game standing still until the advance key goes down, then landing whatever is held
/// differently from the last press, playing `ticks`, drawing, and saying `advanced`.
fn stepped(
    session: &mut Session,
    screen: &Screen,
    heard: &Receiver<Heard>,
    ticks: u32,
) -> Result<(), String> {
    let mut frame = Frame::new();
    let (mut held, mut landed) = (Held::default(), Held::default());
    while session.outcome() == Outcome::Playing {
        match heard.recv() {
            Ok(Heard::Landed(_, action)) => held.apply(action),
            Ok(Heard::Advance) => {
                for action in landed.changes(&held) {
                    session.apply(action);
                }
                landed = held.clone();
                session.lapse(ticks);
                session.draw(&mut frame);
                screen.draw(&frame)?;
                println!("advanced");
                std::io::stdout().flush().map_err(broken)?;
            }
            Ok(Heard::Stop) => return Ok(()),
            Ok(Heard::Gone) | Err(_) => return Err(GONE.into()),
        }
    }
    Ok(())
}

const GONE: &str = "the display stopped answering or refused a request";

/// What is held down and where the pointer is, as the display last said.
#[derive(Clone)]
struct Held {
    keys: [bool; Key::COUNT],
    buttons: [bool; 2],
    pointer: Option<(f32, f32)>,
}

impl Default for Held {
    fn default() -> Held {
        Held { keys: [false; Key::COUNT], buttons: [false; 2], pointer: None }
    }
}

impl Held {
    fn apply(&mut self, action: Action) {
        match action {
            Action::Down(key) => self.keys[key as usize] = true,
            Action::Up(key) => self.keys[key as usize] = false,
            Action::Press { button, x, y } => {
                self.buttons[button as usize] = true;
                self.pointer = Some((x, y));
            }
            Action::Release { button, x, y } => {
                self.buttons[button as usize] = false;
                self.pointer = Some((x, y));
            }
            Action::Move { x, y } | Action::Click { x, y, .. } => self.pointer = Some((x, y)),
            Action::Tap(_) | Action::Wait => {}
        }
    }

    /// The actions that turn this into `now`: the pointer's move, then every key and button
    /// that is held differently.
    fn changes(&self, now: &Held) -> Vec<Action> {
        let mut changes = Vec::new();
        let (x, y) = now.pointer.unwrap_or_default();
        if now.pointer != self.pointer {
            changes.push(Action::Move { x, y });
        }
        for (key, _) in minigame::input::KEYS {
            match (self.keys[key as usize], now.keys[key as usize]) {
                (false, true) => changes.push(Action::Down(key)),
                (true, false) => changes.push(Action::Up(key)),
                _ => {}
            }
        }
        for button in [Button::Left, Button::Right] {
            match (self.buttons[button as usize], now.buttons[button as usize]) {
                (false, true) => changes.push(Action::Press { button, x, y }),
                (true, false) => changes.push(Action::Release { button, x, y }),
                _ => {}
            }
        }
        changes
    }
}

/// The virtual display, stopped when this is dropped.
struct Server(Child);

impl Server {
    /// Starts `Xvfb` the size of the picture, with no key repeat, no network, and no shared
    /// memory, and waits until it says it is ready.
    fn start() -> Result<Server, String> {
        let size = format!("{WIDTH}x{HEIGHT}x24");
        let mut child = Command::new("Xvfb")
            .args([DISPLAY, "-screen", "0", &size, "-r", "-nolisten", "tcp"])
            .args(["-extension", "MIT-SHM", "-displayfd", "1"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("could not start Xvfb, which the game image carries: {e}"))?;
        let mut said = String::new();
        let told = child.stdout.take().map(|out| BufReader::new(out).read_line(&mut said));
        let server = Server(child);
        match told {
            Some(Ok(_)) if !said.trim().is_empty() => Ok(server),
            _ => Err("Xvfb stopped before it was ready".into()),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// A window covering the whole display, and what it takes to draw into it.
struct Screen {
    connection: Arc<RustConnection>,
    window: Window,
    gc: Gcontext,
    rows: usize,
}

impl Screen {
    fn open() -> Result<Screen, String> {
        let (connection, number) = RustConnection::connect(Some(DISPLAY)).map_err(broken)?;
        let setup = connection.setup();
        let root = &setup.roots[number];
        let packed = setup.pixmap_formats.iter().any(|f| f.depth == 24 && f.bits_per_pixel == 32);
        if (root.width_in_pixels, root.height_in_pixels) != (WIDTH as u16, HEIGHT as u16)
            || root.root_depth != 24
            || !packed
            || setup.image_byte_order != ImageOrder::LSB_FIRST
        {
            return Err(format!(
                "the display is not the {WIDTH} by {HEIGHT}, 24-bit screen the game draws"
            ));
        }
        let window = connection.generate_id().map_err(broken)?;
        let listening = EventMask::KEY_PRESS
            | EventMask::KEY_RELEASE
            | EventMask::BUTTON_PRESS
            | EventMask::BUTTON_RELEASE
            | EventMask::POINTER_MOTION;
        connection
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                window,
                root.root,
                0,
                0,
                WIDTH as u16,
                HEIGHT as u16,
                0,
                WindowClass::INPUT_OUTPUT,
                COPY_FROM_PARENT,
                &CreateWindowAux::new().override_redirect(1).event_mask(listening),
            )
            .map_err(broken)?;
        connection.map_window(window).map_err(broken)?;
        connection
            .set_input_focus(InputFocus::POINTER_ROOT, window, CURRENT_TIME)
            .map_err(broken)?;
        let gc = connection.generate_id().map_err(broken)?;
        connection.create_gc(gc, window, &CreateGCAux::new()).map_err(broken)?;
        let row = WIDTH as usize * 4;
        let rows = (connection.maximum_request_bytes().saturating_sub(64) / row).max(1);
        Ok(Screen { connection: Arc::new(connection), window, gc, rows })
    }

    /// The frame onto the screen, in as few requests as the display takes.
    fn draw(&self, frame: &Frame) -> Result<(), String> {
        let bytes: Vec<u8> = frame.pixels().iter().flat_map(|pixel| pixel.to_le_bytes()).collect();
        let row = WIDTH as usize * 4;
        for (band, chunk) in bytes.chunks(row * self.rows).enumerate() {
            self.connection
                .put_image(
                    ImageFormat::Z_PIXMAP,
                    self.window,
                    self.gc,
                    WIDTH as u16,
                    (chunk.len() / row) as u16,
                    0,
                    (band * self.rows) as i16,
                    0,
                    24,
                    chunk,
                )
                .map_err(broken)?;
        }
        self.connection.flush().map_err(broken)
    }
}

/// Every key, button, and movement the display reports, as an action sent with the moment it
/// was read. A key is read against the keyboard as it is mapped at that moment. An error from
/// the display ends the listening, and so the round.
fn listen(connection: &RustConnection, sender: &Sender<Heard>) {
    let Ok(mut keymap) = Keymap::read(connection) else { return };
    while let Ok(event) = connection.wait_for_event() {
        let at = |x: i16, y: i16| (f32::from(x), f32::from(y));
        let action = match event {
            Event::KeyPress(e) if keymap.sym(e.detail) == Some(ADVANCE) => {
                if sender.send(Heard::Advance).is_err() {
                    return;
                }
                None
            }
            Event::KeyPress(e) => keymap.key(e.detail).map(Action::Down),
            Event::KeyRelease(e) => keymap.key(e.detail).map(Action::Up),
            Event::ButtonPress(e) => button(e.detail).map(|button| {
                let (x, y) = at(e.event_x, e.event_y);
                Action::Press { button, x, y }
            }),
            Event::ButtonRelease(e) => button(e.detail).map(|button| {
                let (x, y) = at(e.event_x, e.event_y);
                Action::Release { button, x, y }
            }),
            Event::MotionNotify(e) => {
                let (x, y) = at(e.event_x, e.event_y);
                Some(Action::Move { x, y })
            }
            Event::MappingNotify(e) if e.request == Mapping::KEYBOARD => {
                let Ok(fresh) = Keymap::read(connection) else { return };
                keymap = fresh;
                None
            }
            Event::Error(_) => return,
            _ => None,
        };
        if let Some(action) = action
            && sender.send(Heard::Landed(Instant::now(), action)).is_err()
        {
            return;
        }
    }
}

/// Which keysyms each keycode carries.
struct Keymap {
    first: u8,
    per: usize,
    syms: Vec<Keysym>,
}

impl Keymap {
    fn read(connection: &RustConnection) -> Result<Keymap, String> {
        let setup = connection.setup();
        let (first, last) = (setup.min_keycode, setup.max_keycode);
        let reply = connection
            .get_keyboard_mapping(first, last - first + 1)
            .map_err(broken)?
            .reply()
            .map_err(broken)?;
        Ok(Keymap { first, per: usize::from(reply.keysyms_per_keycode), syms: reply.keysyms })
    }

    /// The first keysym a keycode carries.
    fn sym(&self, code: Keycode) -> Option<Keysym> {
        let from = usize::from(code.checked_sub(self.first)?) * self.per;
        let row = self.syms.get(from..from + self.per)?;
        row.iter().copied().find(|&sym| sym != 0)
    }

    /// The game's key for a keycode, by the first keysym it carries.
    fn key(&self, code: Keycode) -> Option<Key> {
        key(self.sym(code)?)
    }
}

/// The game's key for a keysym, whichever case a letter is in.
fn key(sym: Keysym) -> Option<Key> {
    match sym {
        0x20 => Some(Key::Space),
        0x30..=0x39 | 0x41..=0x5a | 0x61..=0x7a => Key::parse(&char::from(sym as u8).to_string()),
        0xff0d | 0xff8d => Some(Key::Enter),
        0xff08 => Some(Key::Backspace),
        0xff1b => Some(Key::Escape),
        0xff51 => Some(Key::Left),
        0xff52 => Some(Key::Up),
        0xff53 => Some(Key::Right),
        0xff54 => Some(Key::Down),
        0xffe1 | 0xffe2 => Some(Key::Shift),
        _ => None,
    }
}

fn button(detail: u8) -> Option<Button> {
    match detail {
        1 => Some(Button::Left),
        3 => Some(Button::Right),
        _ => None,
    }
}

fn broken(error: impl std::fmt::Display) -> String {
    format!("the display: {error}")
}

#[cfg(test)]
#[path = "tests/screen.rs"]
mod tests;
