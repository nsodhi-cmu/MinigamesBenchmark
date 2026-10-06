//! The games in a browser: the shell behind a handful of plain exports, fed
//! lines of the action language and read back as bytes a canvas can take.

use minigame::frame::{Frame, HEIGHT, WIDTH};
use minigame::input::Action;
use minigame::shell::Shell;
use std::cell::RefCell;

const LINE: usize = 64;

struct Page {
    shell: Shell,
    frame: Frame,
    rgba: Vec<u8>,
    line: [u8; LINE],
}

thread_local! {
    static PAGE: RefCell<Option<Page>> = const { RefCell::new(None) };
}

fn with<T: Default>(f: impl FnOnce(&mut Page) -> T) -> T {
    PAGE.with_borrow_mut(|page| page.as_mut().map(f).unwrap_or_default())
}

#[unsafe(no_mangle)]
pub extern "C" fn width() -> u32 {
    WIDTH
}

#[unsafe(no_mangle)]
pub extern "C" fn height() -> u32 {
    HEIGHT
}

/// The menu, on a fresh shell, replacing whatever was open.
#[unsafe(no_mangle)]
pub extern "C" fn start(seed: u64) {
    let page = Page {
        shell: Shell::new(None, seed, minigame::TICK_HZ as f32, false)
            .expect("the menu always opens"),
        frame: Frame::new(),
        rgba: vec![0; (WIDTH * HEIGHT * 4) as usize],
        line: [0; LINE],
    };
    PAGE.with_borrow_mut(|slot| *slot = Some(page));
}

/// Where a line of the action language is written before `act`, and how long
/// it may be.
#[unsafe(no_mangle)]
pub extern "C" fn line() -> *mut u8 {
    with(|page| page.line.as_mut_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn line_length() -> u32 {
    LINE as u32
}

/// The first `length` bytes of the line, as one action. A line that does not
/// read does nothing.
#[unsafe(no_mangle)]
pub extern "C" fn act(length: u32) {
    with(|page| {
        let text = str::from_utf8(&page.line[..(length as usize).min(LINE)]).unwrap_or("");
        if let Ok(action) = Action::parse(text) {
            page.shell.act(action);
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn advance(seconds: f32) {
    with(|page| {
        page.shell.advance(seconds);
    })
}

/// Whether escape has closed the shell, after which `start` opens another.
#[unsafe(no_mangle)]
pub extern "C" fn closed() -> bool {
    with(|page| !page.shell.open())
}

/// The screen as it is now, `width * height` pixels of red, green, blue, and
/// alpha bytes.
#[unsafe(no_mangle)]
pub extern "C" fn draw() -> *const u8 {
    with(|page| {
        page.shell.draw(&mut page.frame);
        for (pixel, out) in page.frame.pixels().iter().zip(page.rgba.as_chunks_mut::<4>().0) {
            *out = [(pixel >> 16) as u8, (pixel >> 8) as u8, *pixel as u8, 255];
        }
        page.rgba.as_ptr()
    })
}
