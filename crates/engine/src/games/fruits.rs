//! Move a bucket along the ground to catch falling fruit until the quota is met.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "fruits",
    title: "Floundering Fruits",
    brief: "Fruit falls from the top of the screen, some straight down, some at a slant, and \
            some swaying from side to side, each at its own speed. Move the bucket along the \
            ground to catch twenty of them. A fruit is caught when it drops into the mouth of \
            the bucket, and a fruit that misses is simply gone. No fruit ever falls toward where \
            the bucket is when that fruit appears, so the bucket has to keep moving. What kind \
            of fruit it is makes no difference.",
    controls: &["LEFT and RIGHT move the bucket"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Friendly, Tag::Timing],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Fruits::new(seed))
}

pub(crate) const QUOTA: usize = 20;
/// Where the mouth of the bucket is, and half its width.
pub(crate) const RIM: f32 = 440.0;
pub(crate) const BUCKET_HALF: f32 = 22.0;
const BUCKET_TALL: f32 = 28.0;
pub(crate) const BUCKET_PACE: f32 = 360.0;
pub(crate) const FRUIT: f32 = 10.0;
/// The nearest a fruit lands to where the bucket was when it appeared.
pub(crate) const AWAY: f32 = 100.0;
const TOP: f32 = -FRUIT;

const SKINS: [Rgb; 3] = [Rgb(226, 58, 58), Rgb(246, 150, 40), Rgb(150, 84, 200)];
const STEM: Rgb = Rgb(120, 84, 50);
const LEAF: Rgb = Rgb(84, 180, 90);
const BUCKET: Rgb = Rgb(150, 160, 178);

/// A fruit's path, fixed when it appears: from `from` across the top to
/// `land` at the rim over `fall` ticks, swaying `sway` pixels either side a
/// whole number of times so that it still arrives at `land`.
#[derive(Clone, Copy)]
pub(crate) struct Fruit {
    pub(crate) from: f32,
    pub(crate) land: f32,
    pub(crate) fall: u32,
    pub(crate) age: u32,
    pub(crate) sway: f32,
    waves: u32,
    skin: usize,
}

impl Fruit {
    pub(crate) fn at(&self) -> (f32, f32) {
        let along = self.age as f32 / self.fall as f32;
        let wave = libm::sinf(along * std::f32::consts::PI * self.waves as f32);
        (self.from + (self.land - self.from) * along + self.sway * wave, TOP + (RIM - TOP) * along)
    }
}

pub(crate) struct Fruits {
    rng: Rng,
    pub(crate) bucket: f32,
    pub(crate) fruits: Vec<Fruit>,
    pub(crate) caught: usize,
    /// Ticks until the next fruit appears.
    next: u32,
}

impl Fruits {
    pub(crate) fn new(seed: u64) -> Self {
        Fruits {
            rng: Rng::new(seed),
            bucket: frame::WIDTH as f32 * 0.5,
            fruits: Vec::new(),
            caught: 0,
            next: 20,
        }
    }

    fn drop_one(&mut self) {
        let edge = BUCKET_HALF;
        let right = frame::WIDTH as f32 - edge;
        // Where a fruit may land is the ground on either side of the band around
        // the bucket. Drawing along the two stretches end to end picks evenly
        // over both without ever drawing a place that has to be thrown away.
        let left = (self.bucket - AWAY - edge).max(0.0);
        let far = (right - self.bucket - AWAY).max(0.0);
        let along = self.rng.range(0.0, left + far);
        let land = if along < left { edge + along } else { self.bucket + AWAY + (along - left) };
        let (from, sway, waves) = match self.rng.below(3) {
            0 => (land, 0.0, 0),
            1 => (self.rng.range(edge, right), 0.0, 0),
            _ => {
                let sway = self.rng.range(30.0, 60.0).min(land - edge).min(right - land);
                (land, sway, 2 + self.rng.below(3) as u32)
            }
        };
        let fall = 110 + self.rng.below(90) as u32;
        let skin = self.rng.below(SKINS.len());
        self.fruits.push(Fruit { from, land, fall, age: 0, sway, waves, skin });
    }
}

impl Game for Fruits {
    fn update(&mut self, input: &Input) -> Outcome {
        let heading = input.axis(Key::Left, Key::Right);
        self.bucket = (self.bucket + heading * BUCKET_PACE * TICK)
            .clamp(BUCKET_HALF, frame::WIDTH as f32 - BUCKET_HALF);

        self.next -= 1;
        if self.next == 0 {
            self.drop_one();
            self.next = 40 + self.rng.below(30) as u32;
        }

        let bucket = self.bucket;
        let mut caught = 0;
        self.fruits.retain_mut(|fruit| {
            fruit.age += 1;
            if fruit.age == fruit.fall && (fruit.at().0 - bucket).abs() <= BUCKET_HALF {
                caught += 1;
                return false;
            }
            fruit.at().1 < frame::HEIGHT as f32 + FRUIT
        });
        self.caught += caught;
        if self.caught >= QUOTA { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let ground = RIM + BUCKET_TALL;
        frame.rect(0.0, ground, frame::WIDTH as f32, frame::HEIGHT as f32 - ground, palette::PANEL);
        for fruit in &self.fruits {
            let (x, y) = fruit.at();
            frame.disc(x, y, FRUIT, SKINS[fruit.skin]);
            frame.rect(x - 1.0, y - FRUIT - 4.0, 2.0, 5.0, STEM);
            frame.disc(x + 4.0, y - FRUIT - 1.0, 3.0, LEAF);
        }
        let x = self.bucket;
        frame.poly(
            &[
                [x - BUCKET_HALF, RIM],
                [x + BUCKET_HALF, RIM],
                [x + BUCKET_HALF - 6.0, ground],
                [x - BUCKET_HALF + 6.0, ground],
            ],
            BUCKET,
        );
        frame.rect(x - BUCKET_HALF - 2.0, RIM - 2.0, BUCKET_HALF * 2.0 + 4.0, 4.0, palette::INK);
    }
}

#[cfg(test)]
#[path = "../tests/games/fruits.rs"]
mod tests;
