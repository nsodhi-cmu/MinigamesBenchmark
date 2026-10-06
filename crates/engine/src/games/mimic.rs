//! Watch a sequence of panels light up, then give it back.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "mimic",
    title: "Memory Mimic",
    brief: "Four panels light up one after another. When they stop, click them back in \
            the order they lit. Get it right and the sequence grows by one and plays again \
            from the start, beginning at two and ending at six. One wrong panel ends the \
            round.",
    controls: &["left click a panel"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Memory],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Mimic::new(seed))
}

const PANELS: usize = 4;
const OPENING: usize = 2;
const TARGET: usize = 6;
const BEAT: u32 = 54;
const LIT: u32 = 27;
const DARK: u32 = 10;
const WIDE: f32 = 210.0;
const TALL: f32 = 158.0;
const GAP: f32 = 16.0;

enum Phase {
    /// A moment of quiet before a sequence plays, so it does not begin the
    /// instant the last panel of the last one is clicked.
    Waiting {
        tick: u32,
    },
    Showing {
        step: usize,
        tick: u32,
    },
    Listening {
        given: usize,
    },
}

struct Mimic {
    rng: Rng,
    order: Vec<usize>,
    phase: Phase,
}

/// The next panel to flash, never the one that just flashed: two flashes of
/// one panel are told apart only by the dark between them, which a player
/// looking every so often rather than every tick does not see at all.
fn after(rng: &mut Rng, last: Option<usize>) -> usize {
    match last {
        Some(last) => (last + 1 + rng.below(PANELS - 1)) % PANELS,
        None => rng.below(PANELS),
    }
}

impl Mimic {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut order: Vec<usize> = Vec::with_capacity(OPENING);
        for _ in 0..OPENING {
            order.push(after(&mut rng, order.last().copied()));
        }
        Mimic { rng, order, phase: Phase::Waiting { tick: 0 } }
    }

    fn corner(panel: usize) -> (f32, f32) {
        let across = 2.0 * WIDE + GAP;
        let down = 2.0 * TALL + GAP;
        let x = frame::WIDTH as f32 * 0.5 - across * 0.5;
        let y = (frame::HEIGHT as f32 - down) * 0.5;
        (x + (panel % 2) as f32 * (WIDE + GAP), y + (panel / 2) as f32 * (TALL + GAP))
    }

    fn under(x: f32, y: f32) -> Option<usize> {
        (0..PANELS).find(|panel| {
            let (px, py) = Self::corner(*panel);
            x >= px && y >= py && x <= px + WIDE && y <= py + TALL
        })
    }
}

fn tint_of(panel: usize) -> Rgb {
    match panel {
        0 => palette::PLAYER,
        1 => palette::GOOD,
        2 => palette::WARN,
        _ => palette::MARK,
    }
}

/// Each panel carries a shape as well as a color, so neither has to be relied on alone.
fn shape(frame: &mut Frame, panel: usize, cx: f32, cy: f32, tint: Rgb) {
    let r = 40.0;
    match panel {
        0 => frame.disc(cx, cy, r, tint),
        1 => frame.rect(cx - r, cy - r, r * 2.0, r * 2.0, tint),
        2 => frame.wedge(cx, cy, r, r, tint),
        _ => frame.corners(cx, cy, &[r; 4], 0.0, tint),
    }
}

impl Game for Mimic {
    fn update(&mut self, input: &Input) -> Outcome {
        match self.phase {
            Phase::Waiting { tick } => {
                self.phase = if tick + 1 >= BEAT {
                    Phase::Showing { step: 0, tick: 0 }
                } else {
                    Phase::Waiting { tick: tick + 1 }
                };
                Outcome::Playing
            }
            Phase::Showing { step, tick } => {
                let next = tick + 1;
                if next >= LIT + DARK {
                    self.phase = if step + 1 == self.order.len() {
                        Phase::Listening { given: 0 }
                    } else {
                        Phase::Showing { step: step + 1, tick: 0 }
                    };
                } else {
                    self.phase = Phase::Showing { step, tick: next };
                }
                Outcome::Playing
            }
            Phase::Listening { given } => {
                if !input.button_pressed(Button::Left) {
                    return Outcome::Playing;
                }
                let (x, y) = input.cursor();
                let Some(panel) = Mimic::under(x, y) else {
                    return Outcome::Playing;
                };
                if panel != self.order[given] {
                    return Outcome::Lost;
                }
                if given + 1 < self.order.len() {
                    self.phase = Phase::Listening { given: given + 1 };
                    return Outcome::Playing;
                }
                if self.order.len() == TARGET {
                    return Outcome::Won;
                }
                let next = after(&mut self.rng, self.order.last().copied());
                self.order.push(next);
                self.phase = Phase::Waiting { tick: 0 };
                Outcome::Playing
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let showing = match self.phase {
            Phase::Showing { step, tick } if tick < LIT => Some(self.order[step]),
            _ => None,
        };
        for panel in 0..PANELS {
            let (x, y) = Mimic::corner(panel);
            let full = tint_of(panel);
            let alight = showing == Some(panel);
            let back = if alight { full.mix(palette::BACKDROP, 0.55) } else { palette::PANEL };
            frame.rect(x, y, WIDE, TALL, back);
            frame.outline(x, y, WIDE, TALL, 3.0, if alight { palette::INK } else { palette::EDGE });
            let ink = if alight { palette::INK } else { full.mix(palette::BACKDROP, 0.62) };
            shape(frame, panel, x + WIDE * 0.5, y + TALL * 0.5, ink);
        }

        let middle = frame::WIDTH as f32 * 0.5;
        match self.phase {
            Phase::Waiting { .. } => frame.text_center(middle, 12.0, "READY", 2, palette::MUTED),
            Phase::Showing { .. } => frame.text_center(middle, 12.0, "WATCH", 2, palette::MUTED),
            Phase::Listening { given } => {
                frame.text_center(middle, 12.0, "YOUR TURN", 2, palette::WARN);
                let shown = self.order.len() as u32;
                frame.pips(middle, 462.0, shown, given as u32, palette::GOOD);
            }
        }
    }
}
