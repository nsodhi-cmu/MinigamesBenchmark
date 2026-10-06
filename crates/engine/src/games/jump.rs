//! Bounce up a tower of ledges that keeps sinking away beneath you.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "jump",
    title: "Jump Journey",
    brief: "You bounce off every ledge you land on, whether you meant to or not, and the \
            whole tower sinks slowly past you. Steer left and right to line up the next \
            ledge above. You pass up through them and only land coming down. Reach the \
            green ledge at the top; fall off the bottom and the round is over.",
    controls: &["LEFT and RIGHT steer", "the bounce takes care of itself"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Lethal, Tag::Physics, Tag::Platformer],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Jump::new(seed))
}

const ROWS: usize = 24;
const PITCH: f32 = 96.0;
/// The narrowest and widest a ledge is dealt.
const NARROW: f32 = 56.0;
const WIDE: f32 = 104.0;
const LEDGE_TALL: f32 = 12.0;
const WEIGHT: f32 = 1500.0;
const BOUNCE: f32 = 660.0;
const PACE: f32 = 320.0;
const SINK: f32 = 26.0;
const BODY: f32 = 13.0;
/// How far sideways a ledge sits from the one below it, at the least and at
/// the most. A bounce is back down at the next row up after about seven
/// tenths of a second, which carries some two hundred and twenty pixels, so
/// the most is kept well inside that.
const STEP: f32 = 70.0;
const REACH: f32 = 168.0;
const MARGIN: f32 = 46.0;
/// How high up the screen the climb is held, which is what leaves room below
/// to fall back into after a ledge is missed.
const RIDE: f32 = 150.0;

struct Ledge {
    at: (f32, f32),
    wide: f32,
    row: usize,
}

struct Jump {
    ledges: Vec<Ledge>,
    at: (f32, f32),
    fall: f32,
    top: f32,
    highest: usize,
}

fn row_y(row: usize) -> f32 {
    row as f32 * PITCH
}

impl Jump {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut ledges = Vec::new();
        let start = (frame::WIDTH as f32 * 0.5, row_y(ROWS - 1));
        ledges.push(Ledge { at: start, wide: WIDE, row: ROWS - 1 });

        // One ledge a row, always a clear step to one side of the last and
        // inside a bounce of it, and now and then a second one well away from it.
        let edge = frame::WIDTH as f32 - MARGIN;
        let mut chain = start.0;
        for row in (0..ROWS - 1).rev() {
            let side = if chain < MARGIN + REACH {
                1.0
            } else if chain > edge - REACH {
                -1.0
            } else if rng.chance(0.5) {
                1.0
            } else {
                -1.0
            };
            chain = (chain + side * rng.range(STEP, REACH)).clamp(MARGIN, edge);
            let wide = rng.range(NARROW, WIDE);
            ledges.push(Ledge { at: (chain, row_y(row)), wide, row });
            let spare = rng.range(MARGIN, edge);
            if (spare - chain).abs() > wide + WIDE {
                let wide = rng.range(NARROW, WIDE);
                ledges.push(Ledge { at: (spare, row_y(row)), wide, row });
            }
        }

        Jump {
            ledges,
            at: (start.0, start.1 - BODY - LEDGE_TALL * 0.5),
            fall: 0.0,
            top: row_y(ROWS - 1) - RIDE,
            highest: ROWS - 1,
        }
    }

    fn screen(&self, world: f32) -> f32 {
        world - self.top
    }
}

impl Game for Jump {
    fn update(&mut self, input: &Input) -> Outcome {
        let lean = input.axis(Key::Left, Key::Right);
        self.at.0 = (self.at.0 + lean * PACE * TICK).clamp(BODY, frame::WIDTH as f32 - BODY);

        let was = self.at.1;
        self.fall += WEIGHT * TICK;
        self.at.1 += self.fall * TICK;

        if self.fall > 0.0 {
            for ledge in &self.ledges {
                let top = ledge.at.1 - LEDGE_TALL * 0.5;
                let feet_was = was + BODY;
                let feet_now = self.at.1 + BODY;
                if feet_was <= top
                    && feet_now >= top
                    && (self.at.0 - ledge.at.0).abs() <= ledge.wide * 0.5 + BODY * 0.4
                {
                    self.at.1 = top - BODY;
                    self.fall = -BOUNCE;
                    self.highest = self.highest.min(ledge.row);
                    break;
                }
            }
        }

        self.top -= SINK * TICK;
        self.top = self.top.min(self.at.1 - RIDE);

        // Landing on the green ledge, not merely rising past where it is: a
        // bounce off the row below it crosses that line in mid-air.
        if self.highest == 0 {
            return Outcome::Won;
        }
        if self.screen(self.at.1) > frame::HEIGHT as f32 + BODY {
            return Outcome::Lost;
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        for ledge in &self.ledges {
            let y = self.screen(ledge.at.1);
            if y < -20.0 || y > frame::HEIGHT as f32 + 20.0 {
                continue;
            }
            let tint = if ledge.row == 0 { palette::GOOD } else { palette::EDGE };
            frame.rect(
                ledge.at.0 - ledge.wide * 0.5,
                y - LEDGE_TALL * 0.5,
                ledge.wide,
                LEDGE_TALL,
                tint,
            );
            if ledge.row == 0 {
                frame.rect(
                    ledge.at.0 - ledge.wide * 0.5,
                    y - LEDGE_TALL * 0.5 - 5.0,
                    ledge.wide,
                    4.0,
                    palette::INK,
                );
            }
        }
        let y = self.screen(self.at.1);
        frame.disc(self.at.0, y, BODY, palette::PLAYER);
    }
}
