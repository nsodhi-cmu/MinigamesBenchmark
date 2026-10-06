//! Watch something go under, and say where it will come out.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "tunnel",
    title: "Tunnel Tracker",
    brief: "A marker crosses the ground below and passes under the bank, where you cannot \
            see it. Click the mouth you think it will come out of, before it comes out of \
            it. Ten right answers finish the round; one wrong one, or one left too late, \
            ends it.",
    controls: &["left click a mouth"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Lethal, Tag::Quiz, Tag::Timing],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Tunnel::new(seed))
}

const ROUNDS: u32 = 10;
const MOUTHS: usize = 6;
const MOUTH_WIDE: f32 = 76.0;
const BANK_TOP: f32 = 178.0;
/// How a mouth is drawn inside the width it is given, and how deep it is. The
/// hit test reads the same two numbers, so what is live is what is drawn.
const MOUTH_INSET: f32 = 5.0;
const MOUTH_TALL: f32 = 30.0;
const BANK_LOW: f32 = 330.0;
const ENTERS: f32 = 466.0;
const CLIMB: f32 = 120.0;
const REVEAL: u32 = 48;

fn mouth_at(mouth: usize) -> f32 {
    let across = MOUTHS as f32 * MOUTH_WIDE;
    frame::WIDTH as f32 * 0.5 - across * 0.5 + (mouth as f32 + 0.5) * MOUTH_WIDE
}

struct Tunnel {
    rng: Rng,
    from: (f32, f32),
    at: (f32, f32),
    drift: f32,
    leaves: usize,
    said: Option<usize>,
    showing: u32,
    solved: u32,
}

impl Tunnel {
    fn new(seed: u64) -> Self {
        let mut tunnel = Tunnel {
            rng: Rng::new(seed),
            from: (0.0, 0.0),
            at: (0.0, 0.0),
            drift: 0.0,
            leaves: 0,
            said: None,
            showing: 0,
            solved: 0,
        };
        tunnel.lay();
        tunnel
    }

    /// A run that ends squarely inside one mouth rather than on the line between two.
    fn lay(&mut self) {
        self.leaves = self.rng.below(MOUTHS);
        let out = mouth_at(self.leaves) + self.rng.range(-MOUTH_WIDE * 0.22, MOUTH_WIDE * 0.22);
        let start = self.rng.range(70.0, frame::WIDTH as f32 - 70.0);
        let flight = (ENTERS - BANK_TOP) / CLIMB;
        self.from = (start, ENTERS);
        self.at = self.from;
        self.drift = (out - start) / flight;
        self.said = None;
        self.showing = 0;
    }
}

impl Game for Tunnel {
    fn update(&mut self, input: &Input) -> Outcome {
        if self.showing > 0 {
            self.showing -= 1;
            if self.showing == 0 {
                self.solved += 1;
                if self.solved == ROUNDS {
                    return Outcome::Won;
                }
                self.lay();
            }
            return Outcome::Playing;
        }

        self.at.0 += self.drift * TICK;
        self.at.1 -= CLIMB * TICK;

        if input.button_pressed(Button::Left) && self.said.is_none() {
            let (x, y) = input.cursor();
            // Only a mouth as it is drawn is live. The gutter between two of
            // them is backdrop, and a pick made there would be one the player
            // had no way of seeing they had made.
            let picked = (0..MOUTHS).find(|mouth| {
                let left = mouth_at(*mouth) - MOUTH_WIDE * 0.5 + MOUTH_INSET;
                (left..=left + MOUTH_WIDE - MOUTH_INSET * 2.0).contains(&x)
                    && (BANK_TOP - MOUTH_TALL..=BANK_TOP).contains(&y)
            });
            if let Some(picked) = picked {
                if picked != self.leaves {
                    return Outcome::Lost;
                }
                self.said = Some(picked);
                self.showing = REVEAL;
            }
        }

        if self.at.1 <= BANK_TOP && self.said.is_none() {
            return Outcome::Lost;
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        let across = MOUTHS as f32 * MOUTH_WIDE;
        let left = width * 0.5 - across * 0.5;

        // The whole of the line the marker is on, from where it came in to the
        // near edge of the bank. It does not move with the marker: what is
        // asked for is where the line goes, not where the marker has gotten to.
        let along = (self.drift, -CLIMB);
        let fast = libm::hypotf(along.0, along.1);
        let mut step = 0.0;
        while self.from.1 + along.1 / fast * step > BANK_LOW {
            frame.disc(
                self.from.0 + along.0 / fast * step,
                self.from.1 + along.1 / fast * step,
                2.0,
                palette::PLAYER.mix(palette::BACKDROP, 0.45),
            );
            step += 14.0;
        }
        if self.at.1 > BANK_LOW || self.showing > 0 {
            frame.disc(self.at.0, self.at.1, 9.0, palette::PLAYER);
        }

        frame.rect(0.0, BANK_TOP, width, BANK_LOW - BANK_TOP, palette::PANEL);
        frame.rect(0.0, BANK_TOP - 3.0, width, 3.0, palette::EDGE);
        frame.rect(0.0, BANK_LOW, width, 3.0, palette::EDGE);

        for mouth in 0..MOUTHS {
            let x = left + mouth as f32 * MOUTH_WIDE;
            let picked = self.said == Some(mouth);
            let tint = if picked { palette::GOOD } else { palette::EDGE };
            let (bx, bw) = (x + MOUTH_INSET, MOUTH_WIDE - MOUTH_INSET * 2.0);
            frame.bordered(bx, BANK_TOP - MOUTH_TALL, bw, MOUTH_TALL, 3.0, palette::BACKDROP, tint);
            frame.text_center(
                x + MOUTH_WIDE * 0.5,
                BANK_TOP - 22.0,
                &(mouth + 1).to_string(),
                2,
                tint,
            );
        }

        frame.text_center(width * 0.5, 12.0, "WHICH MOUTH", 2, palette::MUTED);
    }
}
