//! Turn tiles over two at a time and remember what was under them.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "pairs",
    title: "Perfect Pairs",
    brief: "Sixteen tiles hide eight pairs of shapes. Turn one over, then another. A \
            matching pair stays face up; anything else stays up until you turn over your \
            next tile. A wrong pair costs nothing but the time. Find all eight pairs.",
    controls: &["left click a tile to turn it over"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Memory, Tag::Explore, Tag::Grid],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Pairs::new(seed))
}

const SIDE: usize = 4;
const TILES: usize = SIDE * SIDE;
const KINDS: usize = TILES / 2;
const WIDE: f32 = 110.0;
const TALL: f32 = 86.0;
const GAP: f32 = 12.0;

/// What is face up: never more than the two a player is allowed.
#[derive(Clone, Copy)]
enum Turned {
    None,
    One(usize),
    Two(usize, usize),
}

impl Turned {
    fn shows(self, slot: usize) -> bool {
        match self {
            Turned::None => false,
            Turned::One(first) => first == slot,
            Turned::Two(first, second) => first == slot || second == slot,
        }
    }
}

struct Pairs {
    faces: [usize; TILES],
    taken: [bool; TILES],
    turned: Turned,
    found: usize,
}

impl Pairs {
    fn new(seed: u64) -> Self {
        let mut faces = [0; TILES];
        for (slot, face) in faces.iter_mut().enumerate() {
            *face = slot % KINDS;
        }
        Rng::new(seed).shuffle(&mut faces);
        Pairs { faces, taken: [false; TILES], turned: Turned::None, found: 0 }
    }

    fn origin() -> (f32, f32) {
        let across = SIDE as f32 * WIDE + (SIDE as f32 - 1.0) * GAP;
        let down = SIDE as f32 * TALL + (SIDE as f32 - 1.0) * GAP;
        (frame::WIDTH as f32 * 0.5 - across * 0.5, (frame::HEIGHT as f32 - down) * 0.5)
    }

    fn corner(slot: usize) -> (f32, f32) {
        let (x, y) = Self::origin();
        (x + (slot % SIDE) as f32 * (WIDE + GAP), y + (slot / SIDE) as f32 * (TALL + GAP))
    }

    fn under(x: f32, y: f32) -> Option<usize> {
        (0..TILES).find(|slot| {
            let (cx, cy) = Self::corner(*slot);
            x >= cx && y >= cy && x <= cx + WIDE && y <= cy + TALL
        })
    }
}

/// The eight shapes, told apart by outline rather than by color alone.
fn shape(frame: &mut Frame, kind: usize, cx: f32, cy: f32, tint: Rgb) {
    let r = 21.0;
    match kind {
        0 => frame.disc(cx, cy, r, tint),
        1 => frame.ring(cx, cy, r, 6.0, tint),
        2 => frame.rect(cx - r, cy - r, r * 2.0, r * 2.0, tint),
        3 => frame.outline(cx - r, cy - r, r * 2.0, r * 2.0, 6.0, tint),
        4 => frame.wedge(cx, cy, r, r, tint),
        5 => frame.wedge(cx, cy, r, -r, tint),
        6 => frame.corners(cx, cy, &[r; 4], 0.0, tint),
        _ => frame.cross(cx, cy, r, 8.0, tint),
    }
}

impl Game for Pairs {
    fn update(&mut self, input: &Input) -> Outcome {
        if !input.button_pressed(Button::Left) {
            return Outcome::Playing;
        }
        let (x, y) = input.cursor();
        let Some(slot) = Pairs::under(x, y) else {
            return Outcome::Playing;
        };
        if self.taken[slot] || self.turned.shows(slot) {
            return Outcome::Playing;
        }
        self.turned = match self.turned {
            Turned::One(first) if self.faces[first] == self.faces[slot] => {
                self.taken[first] = true;
                self.taken[slot] = true;
                self.found += 1;
                Turned::None
            }
            Turned::One(first) => Turned::Two(first, slot),
            Turned::None | Turned::Two(..) => Turned::One(slot),
        };
        if self.found == KINDS { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        for slot in 0..TILES {
            let (x, y) = Pairs::corner(slot);
            let (cx, cy) = (x + WIDE * 0.5, y + TALL * 0.5);
            if self.taken[slot] {
                frame.outline(x, y, WIDE, TALL, 2.0, palette::PANEL);
                shape(frame, self.faces[slot], cx, cy, palette::EDGE);
            } else if self.turned.shows(slot) {
                frame.bordered(x, y, WIDE, TALL, 3.0, palette::PANEL, palette::WARN);
                shape(frame, self.faces[slot], cx, cy, palette::INK);
            } else {
                frame.bordered(x, y, WIDE, TALL, 3.0, palette::PANEL, palette::EDGE);
                frame.rect(cx - 16.0, cy - 2.0, 32.0, 4.0, palette::EDGE);
            }
        }
    }
}
