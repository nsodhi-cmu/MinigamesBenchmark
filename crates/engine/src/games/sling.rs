//! Five buckets, one shot each, and a sling to put the ball in them with.

use crate::aim;
use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "sling",
    title: "Slingshot Sniper",
    brief: "A bucket hangs on a string out across the drop. Hold the left button near the \
            sling, drag back from it and let go, and the ball flies. Drop it into the mouth \
            of the bucket; there is nothing else out there to land on. The dotted line shows \
            the flight the sling is set for. One shot at each of five buckets; miss one, or \
            throw so weakly that the ball never leaves, and the round is over.",
    controls: &["hold the left button near the sling and drag back", "let go to throw"],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Aim, Tag::Physics, Tag::Drag],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Sling::new())
}

pub(crate) const BUCKETS: usize = 5;
/// The top of the little platform the sling stands on. There is no other
/// ground; everything else is a drop.
const LEDGE: f32 = 424.0;
const LEDGE_WIDE: f32 = 112.0;
pub(crate) const POST: (f32, f32) = (150.0, 336.0);
pub(crate) const PULL: f32 = 3.2;
const FASTEST: f32 = 760.0;
/// How near the post a press has to be to take hold of the sling.
const GRAB: f32 = 90.0;
const WEIGHT: f32 = 620.0;
const BALL: f32 = 8.0;
/// How wide the mouth of a bucket is, and how deep it stands.
pub(crate) const MOUTH: f32 = 88.0;
pub(crate) const DEEP: f32 = 48.0;
/// The inside of a bucket.
const STAVE: Rgb = Rgb(64, 54, 46);

/// Where each bucket's mouth sits, left edge first. Every one of them hangs on
/// a string let down from above.
pub(crate) const BUCKETS_SET: [(f32, f32); BUCKETS] =
    [(466.0, 368.0), (348.0, 318.0), (536.0, 372.0), (282.0, 220.0), (498.0, 288.0)];

pub(crate) struct Sling {
    pub(crate) bucket: usize,
    drawing: Option<(f32, f32)>,
    ball: Option<((f32, f32), (f32, f32))>,
    settling: u32,
    landed: bool,
}

impl Sling {
    pub(crate) fn new() -> Self {
        Sling { bucket: 0, drawing: None, ball: None, settling: 0, landed: false }
    }
}

/// The speed the sling would throw at, were it let go of here.
pub(crate) fn cast_from(x: f32, y: f32) -> (f32, f32) {
    aim::cast(POST, (x, y), PULL, FASTEST)
}

/// Where a ball is in its flight.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Flight {
    Flying,
    In,
    Out,
}

/// One tick of the ball's flight, which is the only place the rule for what
/// counts as going in is written: over the mouth, between its sides, and on
/// the way down, as a ball into a bucket is.
pub(crate) fn fly(bucket: usize, at: &mut (f32, f32), pace: &mut (f32, f32)) -> Flight {
    let (left, lip) = BUCKETS_SET[bucket];
    let was = at.1;
    pace.1 += WEIGHT * TICK;
    at.0 += pace.0 * TICK;
    at.1 += pace.1 * TICK;
    if was <= lip && at.1 > lip && at.0 > left + BALL && at.0 < left + MOUTH - BALL {
        return Flight::In;
    }
    let gone = at.1 > frame::HEIGHT as f32 + 40.0
        || at.1 < -400.0
        || at.0 > frame::WIDTH as f32 + 200.0
        || at.0 < -60.0;
    if gone { Flight::Out } else { Flight::Flying }
}

/// Whether a ball thrown this way drops into the bucket for this round.
#[cfg(test)]
pub(crate) fn lands_in(bucket: usize, throw: (f32, f32)) -> bool {
    let (mut at, mut pace) = (POST, throw);
    for _ in 0..900 {
        match fly(bucket, &mut at, &mut pace) {
            Flight::Flying => {}
            Flight::In => return true,
            Flight::Out => return false,
        }
    }
    false
}

impl Game for Sling {
    fn update(&mut self, input: &Input) -> Outcome {
        if self.settling > 0 {
            self.settling -= 1;
            if self.settling == 0 {
                if !self.landed {
                    return Outcome::Lost;
                }
                if self.bucket + 1 == BUCKETS {
                    return Outcome::Won;
                }
                self.bucket += 1;
                self.ball = None;
                self.landed = false;
            }
            return Outcome::Playing;
        }

        if let Some((mut at, mut pace)) = self.ball {
            let how = fly(self.bucket, &mut at, &mut pace);
            self.ball = Some((at, pace));
            match how {
                Flight::Flying => {}
                Flight::In => {
                    self.landed = true;
                    self.settling = 30;
                }
                Flight::Out => {
                    self.ball = None;
                    self.settling = 21;
                }
            }
            return Outcome::Playing;
        }

        if let Some((x, y)) = aim::drawn(input, POST, GRAB, &mut self.drawing) {
            let pace = cast_from(x, y);
            if libm::hypotf(pace.0, pace.1) > 60.0 {
                self.ball = Some((POST, pace));
            } else {
                self.settling = 21;
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        let (left, lip) = BUCKETS_SET[self.bucket];
        frame.rect(
            POST.0 - LEDGE_WIDE * 0.5,
            LEDGE,
            LEDGE_WIDE,
            frame::HEIGHT as f32 - LEDGE,
            palette::PANEL,
        );
        frame.rect(POST.0 - LEDGE_WIDE * 0.5, LEDGE, LEDGE_WIDE, 3.0, palette::EDGE);
        frame.rect(POST.0 - 4.0, POST.1, 8.0, LEDGE - POST.1, palette::EDGE);
        frame.rect(left + MOUTH * 0.5 - 1.0, 0.0, 2.0, lip, palette::EDGE);
        frame.rect(left + 5.0, lip, MOUTH - 10.0, DEEP - 5.0, STAVE);
        // The inside of the mouth, darker than the rest so the bucket reads as
        // open at the top rather than as a block.
        frame.rect(left + 5.0, lip, MOUTH - 10.0, 10.0, STAVE.mix(palette::BACKDROP, 0.55));

        if let Some((x, y)) = self.drawing {
            let pace = cast_from(x, y);
            frame.line(POST.0, POST.1, x, y, 3.0, palette::MUTED);
            // The flight the sling is set for, stopped where the ball would
            // stop, so that a line ending in the bucket is a throw that lands.
            let (mut spot, mut step) = (POST, pace);
            let mut over = false;
            for mark in 0..160 {
                if fly(self.bucket, &mut spot, &mut step) != Flight::Flying {
                    over = true;
                    break;
                }
                if mark % 6 == 0 {
                    frame.disc(spot.0, spot.1, 2.5, palette::EDGE);
                }
            }
            if over {
                frame.disc(spot.0, spot.1, 3.5, palette::MUTED);
            }
        }
        if let Some((at, _)) = self.ball {
            frame.disc(at.0, at.1, BALL, palette::PLAYER);
        }

        // The bucket goes over the ball, so one dropping in passes behind the
        // near side of it as a ball into a bucket does.
        frame.rect(left, lip - 5.0, 5.0, DEEP + 5.0, palette::MUTED);
        frame.rect(left + MOUTH - 5.0, lip - 5.0, 5.0, DEEP + 5.0, palette::MUTED);
        frame.rect(left, lip + DEEP - 5.0, MOUTH, 5.0, palette::MUTED);
        frame.rect(left + 5.0, lip + DEEP - 12.0, MOUTH - 10.0, 7.0, palette::GOOD);

        frame.disc(POST.0, POST.1, 6.0, palette::MUTED);
        frame.text_center(width * 0.5, 8.0, "DRAG BACK FROM THE SLING", 1, palette::MUTED);
    }
}

#[cfg(test)]
#[path = "../tests/games/sling.rs"]
mod tests;
