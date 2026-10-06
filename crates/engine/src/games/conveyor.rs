//! Take the thing that is asked for off the belt, ten times over.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "conveyor",
    title: "Conveyor Collect",
    brief: "The belt never stops, and what is on it comes down any way around. What is \
            wanted is shown above it; click one of those as it goes past and the next thing \
            is asked for instead. The crossed pieces are scrap and are never wanted. Click \
            anything but what is asked for and the round is over, though clicking the bare \
            belt costs nothing. Ten of them finish it.",
    controls: &["left click a piece on the belt"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Aim, Tag::Timing],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Conveyor::new(seed))
}

/// The pieces worth taking. Scrap comes after them and is never asked for.
const KINDS: usize = 5;
const SCRAP: usize = KINDS;
const QUOTA: u32 = 10;
const BELT_TOP: f32 = 238.0;
const BELT_TALL: f32 = 124.0;
const PACE: f32 = 148.0;
const APART: f32 = 62.0;
const PIECE: f32 = 16.0;
/// How near a click has to be to take a piece, which is also how far past the
/// screen edge a piece is still drawn, so that nothing can be taken unseen.
const REACH: f32 = PIECE + 3.0;
/// How far from the middle of the belt a piece may sit.
const STRAY: f32 = BELT_TALL * 0.5 - PIECE - 10.0;

const TINTS: [Rgb; KINDS + 1] =
    [palette::PLAYER, palette::GOOD, palette::WARN, palette::MARK, palette::PINK, palette::MUTED];

struct Piece {
    at: (f32, f32),
    kind: usize,
    spin: f32,
}

struct Conveyor {
    rng: Rng,
    age: u32,
    pieces: Vec<Piece>,
    wanted: usize,
    taken: u32,
}

impl Conveyor {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let wanted = rng.below(KINDS);
        let mut belt = Conveyor { rng, age: 0, pieces: Vec::new(), wanted, taken: 0 };
        let mut x = 0.0;
        while x < frame::WIDTH as f32 + APART {
            let kind = belt.rng.below(KINDS + 1);
            belt.lay(x, kind);
            x += APART + belt.rng.range(-14.0, 14.0);
        }
        belt
    }

    fn lay(&mut self, x: f32, kind: usize) {
        let stray = self.rng.range(-STRAY, STRAY);
        let spin = self.rng.range(0.0, std::f32::consts::TAU);
        self.pieces.push(Piece { at: (x, self.middle() + stray), kind, spin });
    }

    /// Something other than what was just taken, so no two in a row are alike.
    fn ask_again(&mut self) {
        let step = 1 + self.rng.below(KINDS - 1);
        self.wanted = (self.wanted + step) % KINDS;
    }

    fn middle(&self) -> f32 {
        BELT_TOP + BELT_TALL * 0.5
    }
}

/// Six pieces, each its own color and its own count of corners, so that one
/// turned any way around is still the one it was.
fn piece(frame: &mut Frame, kind: usize, cx: f32, cy: f32, r: f32, spin: f32) {
    let tint = TINTS[kind];
    match kind {
        0 => frame.disc(cx, cy, r, tint),
        1 => frame.corners(cx, cy, &[r; 3], spin, tint),
        2 => frame.corners(cx, cy, &[r; 4], spin, tint),
        3 => frame.corners(cx, cy, &[r; 5], spin, tint),
        4 => frame.corners(cx, cy, &[r, r * 0.45].repeat(5), spin, tint),
        _ => frame.crossed(cx, cy, r, r * 0.42, spin, tint),
    }
}

impl Game for Conveyor {
    fn update(&mut self, input: &Input) -> Outcome {
        self.age += 1;
        for held in &mut self.pieces {
            held.at.0 -= PACE * TICK;
        }
        self.pieces.retain(|held| held.at.0 > -APART);
        let tail = self.pieces.iter().fold(0.0f32, |far, held| far.max(held.at.0));
        if tail < frame::WIDTH as f32 + APART {
            // Ask for what is wanted often enough that it is never a long wait,
            // and let scrap through often enough that it is always in the way.
            let kind = if self.rng.chance(0.3) {
                self.wanted
            } else if self.rng.chance(0.3) {
                SCRAP
            } else {
                self.rng.below(KINDS)
            };
            let gap = APART + self.rng.range(-14.0, 14.0);
            self.lay(tail + gap, kind);
        }

        if input.button_pressed(Button::Left) {
            let (x, y) = input.cursor();
            // Whichever is nearest the click, so a piece behind another one
            // cannot be taken for it.
            let hit = self
                .pieces
                .iter()
                .enumerate()
                .map(|(slot, held)| (slot, libm::hypotf(held.at.0 - x, held.at.1 - y)))
                .filter(|(_, away)| *away <= REACH)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(slot, _)| slot);
            if let Some(slot) = hit {
                if self.pieces[slot].kind != self.wanted {
                    return Outcome::Lost;
                }
                self.pieces.remove(slot);
                self.taken += 1;
                if self.taken == QUOTA {
                    return Outcome::Won;
                }
                self.ask_again();
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        frame.rect(0.0, BELT_TOP, width, BELT_TALL, palette::PANEL);
        frame.rect(0.0, BELT_TOP, width, 3.0, palette::EDGE);
        frame.rect(0.0, BELT_TOP + BELT_TALL - 3.0, width, 3.0, palette::EDGE);
        let roll = -(self.age as f32 * TICK * PACE).rem_euclid(28.0);
        let mut tread = roll;
        while tread < width {
            frame.rect(tread, BELT_TOP + BELT_TALL - 12.0, 14.0, 6.0, palette::EDGE);
            tread += 28.0;
        }

        for held in &self.pieces {
            if held.at.0 < -REACH || held.at.0 > width + REACH {
                continue;
            }
            piece(frame, held.kind, held.at.0, held.at.1, PIECE, held.spin);
        }

        frame.text_center(width * 0.5, 22.0, "WANTED", 2, palette::MUTED);
        piece(frame, self.wanted, width * 0.5, 86.0, 26.0, 0.0);

        frame.pips(width * 0.5, 430.0, QUOTA, self.taken, palette::GOOD);
    }
}
