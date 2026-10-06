//! Swing along a line of vines, letting go at the right moment each time.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "jungle",
    title: "Jungle Joyride",
    brief: "Ten vines hang between where you start and the green line you are making for. \
            Hold SPACE to keep hold of a vine and let go to fly; you take hold of whatever \
            is in reach the moment you press it again. LEFT and RIGHT swing you harder \
            while you are holding on. Fall in the water and the round is over.",
    controls: &["hold SPACE to hold on, release to let go", "LEFT and RIGHT swing you"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Hostile, Tag::Physics, Tag::Timing],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Jungle::new())
}

const VINES: usize = 10;
const FIRST: f32 = 250.0;
const SPACING: f32 = 214.0;
const HANG: f32 = 88.0;
const LONGEST: f32 = 186.0;
const SHORTEST: f32 = 64.0;
const WEIGHT: f32 = 1100.0;
const PUMP: f32 = 4.6;
const GRAB: f32 = 58.0;
const BODY: f32 = 12.0;
/// The surface of the water, which is drawn, so that how far you may fall is
/// something the picture says rather than something you find out by falling.
const DROWNS: f32 = 440.0;
const WATER: Rgb = Rgb(38, 76, 104);

fn anchor(vine: usize) -> (f32, f32) {
    (FIRST + vine as f32 * SPACING, HANG)
}

/// The line to cross, as far past the last vine as the vines are apart.
fn landing() -> f32 {
    FIRST + VINES as f32 * SPACING
}

enum Hold {
    On { vine: usize, rope: f32, angle: f32, swing: f32 },
    Off { speed: (f32, f32) },
}

struct Jungle {
    at: (f32, f32),
    hold: Hold,
    reached: usize,
    /// You keep hold of the first vine until you have asked to let go of it, so
    /// the round does not drop you before you have reached for the key.
    begun: bool,
}

impl Jungle {
    fn new() -> Self {
        let rope = LONGEST;
        // Hanging straight down and quite still: the swing is yours to build.
        let angle = 0.0f32;
        let (ax, ay) = anchor(0);
        Jungle {
            at: (ax + rope * libm::sinf(angle), ay + rope * libm::cosf(angle)),
            hold: Hold::On { vine: 0, rope, angle, swing: 0.0 },
            reached: 0,
            begun: false,
        }
    }

    /// The vine in reach whose anchor is nearest, and how far the player is from that anchor.
    fn nearest(&self) -> Option<(usize, f32)> {
        (0..VINES)
            .filter_map(|vine| {
                let (ax, ay) = anchor(vine);
                let down = (self.at.1 - ay).clamp(0.0, LONGEST);
                let gap = libm::hypotf(self.at.0 - ax, self.at.1 - ay - down);
                (gap <= GRAB).then_some((vine, libm::hypotf(self.at.0 - ax, self.at.1 - ay)))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }
}

impl Game for Jungle {
    fn update(&mut self, input: &Input) -> Outcome {
        self.begun |= input.pressed(Key::Space);
        let holding = input.held(Key::Space) || !self.begun;
        let lean = input.axis(Key::Left, Key::Right);

        match self.hold {
            Hold::On { vine, rope, mut angle, mut swing } => {
                if !holding {
                    let (sin, cos) = libm::sincosf(angle);
                    let speed = (swing * rope * cos, -swing * rope * sin);
                    self.hold = Hold::Off { speed };
                } else {
                    swing += (-(WEIGHT / rope) * libm::sinf(angle) + lean * PUMP) * TICK;
                    swing = swing.clamp(-4.2, 4.2);
                    angle += swing * TICK;
                    let (ax, ay) = anchor(vine);
                    self.at = (ax + rope * libm::sinf(angle), ay + rope * libm::cosf(angle));
                    self.hold = Hold::On { vine, rope, angle, swing };
                    self.reached = self.reached.max(vine);
                }
            }
            Hold::Off { mut speed } => {
                speed.1 += WEIGHT * TICK;
                self.at.0 += speed.0 * TICK;
                self.at.1 += speed.1 * TICK;
                self.hold = Hold::Off { speed };
                if holding && let Some((vine, rope)) = self.nearest() {
                    let (ax, ay) = anchor(vine);
                    let rope = rope.clamp(SHORTEST, LONGEST);
                    let angle = libm::atan2f(self.at.0 - ax, self.at.1 - ay);
                    let (sin, cos) = libm::sincosf(angle);
                    let swing = (speed.0 * cos - speed.1 * sin) / rope;
                    self.at = (ax + rope * libm::sinf(angle), ay + rope * libm::cosf(angle));
                    self.hold = Hold::On { vine, rope, angle, swing };
                    self.reached = self.reached.max(vine);
                }
            }
        }

        if self.at.0 >= landing() {
            return Outcome::Won;
        }
        if self.at.1 > DROWNS { Outcome::Lost } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let camera = (self.at.0 - 220.0).max(0.0);
        let at = |x: f32| x - camera;

        frame.rect(at(landing()) - 3.0, 0.0, 6.0, frame::HEIGHT as f32, palette::GOOD);

        let gripped = match self.hold {
            Hold::On { vine, .. } => Some(vine),
            Hold::Off { .. } => None,
        };
        for vine in 0..VINES {
            let (ax, ay) = anchor(vine);
            let x = at(ax);
            if !(-40.0..frame::WIDTH as f32 + 40.0).contains(&x) {
                continue;
            }
            let tint = if gripped == Some(vine) { palette::GOOD } else { palette::MUTED };
            frame.rect(x - 14.0, 0.0, 28.0, ay, palette::PANEL);
            match self.hold {
                Hold::On { vine: held, rope, angle, .. } if held == vine => {
                    let tip = (ax + rope * libm::sinf(angle), ay + rope * libm::cosf(angle));
                    frame.line(x, ay, at(tip.0), tip.1, 4.0, tint);
                }
                _ => frame.line(x, ay, x, ay + LONGEST, 4.0, tint),
            }
            frame.disc(x, ay, 8.0, palette::EDGE);
        }

        frame.rect(0.0, DROWNS, frame::WIDTH as f32, frame::HEIGHT as f32 - DROWNS, WATER);
        frame.rect(0.0, DROWNS, frame::WIDTH as f32, 2.0, palette::MUTED);
        frame.disc(at(self.at.0), self.at.1, BODY, palette::PLAYER);
    }
}
