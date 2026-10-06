//! The night and the ship under it, for the games shot from the bottom of the
//! screen.

use crate::frame::{self, Frame};
use crate::palette;

const STARS: usize = 48;
/// Half the ship across.
pub const SHIP_HALF: f32 = 16.0;
/// How far the ship stands above the line it flies along.
pub const SHIP_TALL: f32 = 16.0;

/// A fixed scatter of stars. The same every round, because it is the backdrop
/// rather than anything to read.
pub fn night(frame: &mut Frame) {
    for star in 0..STARS {
        let x = (star * 131 + 37) % frame::WIDTH as usize;
        let y = (star * 89 + star * star * 7) % (frame::HEIGHT as usize - 40);
        frame.rect(x as f32, y as f32, 2.0, 2.0, palette::EDGE);
    }
}

/// The player's ship, pointing up, its base on `y`.
pub fn ship(frame: &mut Frame, x: f32, y: f32) {
    let points = [[x, y - SHIP_TALL], [x + SHIP_HALF, y], [x - SHIP_HALF, y]];
    frame.poly(&points, palette::PLAYER);
}

/// A shot in flight, its top at `y`.
pub fn shot(frame: &mut Frame, x: f32, y: f32) {
    frame.rect(x - 1.5, y, 3.0, 12.0, palette::INK);
}

/// A shot falling back the other way, its bottom at `y`.
pub fn bomb(frame: &mut Frame, x: f32, y: f32) {
    frame.rect(x - 2.0, y - 12.0, 4.0, 12.0, palette::WARN);
}
