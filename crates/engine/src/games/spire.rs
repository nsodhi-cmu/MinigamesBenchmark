//! Move a stack of rings one at a time and never stack a wide one on a narrow one.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "spire",
    title: "Spire Stack",
    brief: "Four rings sit on one post, widest at the bottom. Move them all onto the post \
            marked in green. Click a post to lift its top ring, click another to set it \
            down. A ring will not sit on a narrower one.",
    controls: &["left click a post to lift", "left click a post to drop"],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Spire::new())
}

const RINGS: u8 = 4;
const POSTS: usize = 3;
const BASE: f32 = 396.0;
const POST_HEIGHT: f32 = 170.0;
const RING_HEIGHT: f32 = 26.0;

struct Spire {
    posts: [Vec<u8>; POSTS],
    goal: usize,
    lifted: Option<u8>,
}

impl Spire {
    fn new() -> Self {
        let mut posts = [const { Vec::new() }; POSTS];
        posts[0] = (1..=RINGS).rev().collect();
        Spire { posts, goal: POSTS - 1, lifted: None }
    }

    fn under(x: f32) -> usize {
        ((x / frame::WIDTH as f32 * POSTS as f32) as usize).min(POSTS - 1)
    }

    fn center(post: usize) -> f32 {
        frame::WIDTH as f32 * (post as f32 + 0.5) / POSTS as f32
    }

    fn width_of(ring: u8) -> f32 {
        44.0 + ring as f32 * 30.0
    }
}

impl Game for Spire {
    fn update(&mut self, input: &Input) -> Outcome {
        if input.button_pressed(Button::Left) {
            let post = Self::under(input.cursor().0);
            match self.lifted {
                None => self.lifted = self.posts[post].pop(),
                Some(ring) => {
                    if self.posts[post].last().is_none_or(|top| *top > ring) {
                        self.posts[post].push(ring);
                        self.lifted = None;
                    }
                }
            }
        }
        if self.posts[self.goal].len() == RINGS as usize { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        for post in 0..POSTS {
            let x = Self::center(post);
            frame.rect(x - 90.0, BASE, 180.0, 10.0, palette::PANEL);
            frame.rect(x - 5.0, BASE - POST_HEIGHT, 10.0, POST_HEIGHT, palette::EDGE);
            let tint = if post == self.goal { palette::GOOD } else { palette::EDGE };
            frame.rect(x - 90.0, BASE + 12.0, 180.0, 4.0, tint);

            for (level, ring) in self.posts[post].iter().enumerate() {
                let width = Self::width_of(*ring);
                let y = BASE - (level as f32 + 1.0) * RING_HEIGHT;
                let tint = palette::PLAYER.mix(palette::MARK, *ring as f32 / RINGS as f32);
                frame.rect(x - width * 0.5, y + 2.0, width, RING_HEIGHT - 4.0, tint);
                frame.outline(
                    x - width * 0.5,
                    y + 2.0,
                    width,
                    RING_HEIGHT - 4.0,
                    2.0,
                    palette::INK,
                );
            }
        }

        if let Some(ring) = self.lifted {
            let width = Self::width_of(ring);
            let cx = frame::WIDTH as f32 * 0.5;
            let y = 26.0;
            frame.rect(cx - width * 0.5, y, width, RING_HEIGHT - 4.0, palette::WARN);
            frame.text_center(cx, y - 20.0, "HOLDING", 1, palette::WARN);
        }
    }
}
