//! Walk one numbered tile across the board to the corner it wants.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "slide",
    title: "Slide Sixteen",
    brief: "Fifteen numbered tiles and one empty square. Click a tile beside the empty \
            square and it slides into it. Get tile 1, which starts in the top left, into the \
            bottom right corner.",
    controls: &["left click a tile next to the empty square"],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle, Tag::Grid],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Slide::new())
}

const SIDE: usize = 4;
const CELLS: usize = SIDE * SIDE;
const GOAL: usize = CELLS - 1;
const CELL: f32 = 86.0;
const GAP: f32 = 8.0;
const BLANK: u8 = 0;
const WANTED: u8 = 1;

struct Slide {
    tiles: [u8; CELLS],
}

impl Slide {
    fn new() -> Self {
        let mut tiles = [BLANK; CELLS];
        for (slot, tile) in tiles.iter_mut().enumerate().take(CELLS - 1) {
            *tile = slot as u8 + 1;
        }
        Slide { tiles }
    }

    fn origin() -> (f32, f32) {
        let span = SIDE as f32 * CELL + (SIDE as f32 - 1.0) * GAP;
        (frame::WIDTH as f32 * 0.5 - span * 0.5, (frame::HEIGHT as f32 - span) * 0.5)
    }

    fn corner(slot: usize) -> (f32, f32) {
        let (x, y) = Self::origin();
        (x + (slot % SIDE) as f32 * (CELL + GAP), y + (slot / SIDE) as f32 * (CELL + GAP))
    }

    fn under(x: f32, y: f32) -> Option<usize> {
        (0..CELLS).find(|slot| {
            let (cx, cy) = Self::corner(*slot);
            x >= cx && y >= cy && x <= cx + CELL && y <= cy + CELL
        })
    }

    fn beside(a: usize, b: usize) -> bool {
        let (ax, ay) = (a % SIDE, a / SIDE);
        let (bx, by) = (b % SIDE, b / SIDE);
        ax.abs_diff(bx) + ay.abs_diff(by) == 1
    }
}

impl Game for Slide {
    fn update(&mut self, input: &Input) -> Outcome {
        if input.button_pressed(Button::Left) {
            let (x, y) = input.cursor();
            let empty = self.tiles.iter().position(|tile| *tile == BLANK).expect("one is empty");
            if let Some(slot) = Slide::under(x, y)
                && slot != empty
                && Slide::beside(slot, empty)
            {
                self.tiles.swap(slot, empty);
            }
        }
        if self.tiles[GOAL] == WANTED { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let (ox, oy) = Slide::origin();
        let span = SIDE as f32 * CELL + (SIDE as f32 - 1.0) * GAP;
        frame.outline(ox - 6.0, oy - 6.0, span + 12.0, span + 12.0, 3.0, palette::EDGE);

        for slot in 0..CELLS {
            let tile = self.tiles[slot];
            if tile == BLANK {
                continue;
            }
            let (x, y) = Slide::corner(slot);
            let wanted = tile == WANTED;
            let fill = if wanted { palette::PLAYER } else { palette::PANEL };
            frame.rect(x, y, CELL, CELL, fill);
            frame.outline(x, y, CELL, CELL, 2.0, if wanted { palette::INK } else { palette::EDGE });
            let ink = if wanted { palette::BACKDROP } else { palette::INK };
            frame.text_center(x + CELL * 0.5, y + CELL * 0.5 - 16.0, &tile.to_string(), 4, ink);
        }
        let (gx, gy) = Slide::corner(GOAL);
        frame.outline(gx - 3.0, gy - 3.0, CELL + 6.0, CELL + 6.0, 3.0, palette::GOOD);
    }
}
