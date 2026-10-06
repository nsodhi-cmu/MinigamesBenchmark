//! Four lanes of falling tiles, some of which have to be held down.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "tiles",
    title: "Tile Tech",
    brief: "Tiles fall down four lanes onto the line. Strike D, F, J, or K as the tile for \
            that lane reaches it. A long tile has to be held down until its far end \
            reaches the line, and other lanes will want striking while you hold it. Miss \
            one, let go of one early, or strike a lane with nothing in it, and the round is \
            over. Last the thirty seconds to finish.",
    controls: &["D F J K, one to a lane", "hold a key down for a long tile"],
    pointer: false,
    scoring: Scoring::Boolean,
    varies: true,
    tags: &[Tag::Fixed, Tag::Lethal, Tag::Timing],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Tiles::new(seed))
}

const LANES: usize = 4;
const KEYS: [Key; LANES] = [Key::D, Key::F, Key::J, Key::K];
const LABELS: [&str; LANES] = ["D", "F", "J", "K"];
const LASTS: u32 = 30 * TICK_HZ;
/// How far on either side of the line a strike still counts.
const LEEWAY: u32 = 14;
const BEAT: u32 = 36;
const OPENING: u32 = 4;
const CLOSING: u32 = 50;
/// How many tiles a chart is dealt, out of the beats between opening and closing.
const TILES: usize = 30;
const SCROLL: f32 = 200.0;
const LANE_WIDE: f32 = 84.0;
const LINE: f32 = 396.0;

#[derive(Clone, Copy, PartialEq)]
enum Life {
    Waiting,
    Held,
    Done,
}

struct Tile {
    lane: usize,
    at: u32,
    until: u32,
    life: Life,
}

struct Tiles {
    tiles: Vec<Tile>,
    age: u32,
}

impl Tiles {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut tiles: Vec<Tile> = Vec::new();
        let mut busy = [0u32; LANES];
        let beats: Vec<u32> = (OPENING..CLOSING).collect();
        let mut dealt = rng.take(&beats, TILES);
        dealt.sort_unstable();
        for beat in dealt {
            let at = beat * BEAT;
            let free: Vec<usize> = (0..LANES).filter(|lane| busy[*lane] + BEAT <= at).collect();
            if free.is_empty() {
                continue;
            }
            let lane = free[rng.below(free.len())];
            let long = rng.chance(0.2);
            let held = at + BEAT * (2 + rng.below(3) as u32);
            let until = if long && held + LEEWAY < LASTS { held } else { at };
            busy[lane] = until;
            tiles.push(Tile { lane, at, until, life: Life::Waiting });
        }
        Tiles { tiles, age: 0 }
    }

    fn falling(&self, tick: u32) -> f32 {
        LINE - (tick as f32 - self.age as f32) * SCROLL * TICK
    }
}

impl Game for Tiles {
    fn update(&mut self, input: &Input) -> Outcome {
        self.age += 1;
        let age = self.age;

        for (lane, key) in KEYS.iter().enumerate() {
            if input.pressed(*key) {
                let wanted = self.tiles.iter_mut().find(|tile| {
                    tile.lane == lane
                        && tile.life == Life::Waiting
                        && tile.at.abs_diff(age) <= LEEWAY
                });
                let Some(tile) = wanted else {
                    return Outcome::Lost;
                };
                tile.life = if tile.until > tile.at { Life::Held } else { Life::Done };
            }
            if input.released(*key)
                && let Some(tile) =
                    self.tiles.iter_mut().find(|tile| tile.lane == lane && tile.life == Life::Held)
            {
                if age + LEEWAY < tile.until {
                    return Outcome::Lost;
                }
                tile.life = Life::Done;
            }
        }

        for tile in &self.tiles {
            match tile.life {
                Life::Waiting if age > tile.at + LEEWAY => return Outcome::Lost,
                Life::Held if age > tile.until + LEEWAY => return Outcome::Lost,
                _ => {}
            }
        }

        if self.age >= LASTS { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let left = frame::WIDTH as f32 * 0.5 - LANES as f32 * LANE_WIDE * 0.5;
        for lane in 0..LANES {
            let x = left + lane as f32 * LANE_WIDE;
            let shade = if lane % 2 == 0 { 0.0 } else { 0.35 };
            frame.rect(
                x,
                0.0,
                LANE_WIDE,
                frame::HEIGHT as f32,
                palette::BACKDROP.mix(palette::PANEL, 0.5 + shade),
            );
            frame.rect(x, 0.0, 2.0, frame::HEIGHT as f32, palette::EDGE);
        }
        frame.rect(
            left + LANES as f32 * LANE_WIDE - 2.0,
            0.0,
            2.0,
            frame::HEIGHT as f32,
            palette::EDGE,
        );

        for tile in &self.tiles {
            if tile.life == Life::Done {
                continue;
            }
            let x = left + tile.lane as f32 * LANE_WIDE + 6.0;
            let wide = LANE_WIDE - 12.0;
            let foot = self.falling(tile.at);
            let head = self.falling(tile.until);
            if foot < -60.0 || head > frame::HEIGHT as f32 {
                continue;
            }
            if tile.until > tile.at {
                let tint = if tile.life == Life::Held { palette::GOOD } else { palette::MARK };
                frame.rect(
                    x,
                    head - 9.0,
                    wide,
                    foot - head + 18.0,
                    tint.mix(palette::BACKDROP, 0.45),
                );
                frame.rect(x, head - 9.0, wide, 9.0, tint);
                frame.rect(x, foot, wide, 9.0, tint);
            } else {
                frame.rect(x, foot - 9.0, wide, 18.0, palette::PLAYER);
            }
        }

        frame.rect(left, LINE - 2.0, LANES as f32 * LANE_WIDE, 4.0, palette::WARN);
        for (lane, label) in LABELS.iter().enumerate() {
            let x = left + lane as f32 * LANE_WIDE;
            frame.text_center(x + LANE_WIDE * 0.5, LINE + 18.0, label, 2, palette::MUTED);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/tiles.rs"]
mod tests;
