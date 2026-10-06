//! Cross the lanes without being run down, three times over.

use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "traffic",
    title: "Traffic Trot",
    brief: "Four lanes of traffic run between you and the far curb, each at its own speed \
            and in its own direction, with a strip to stand on between them. Walk up to the far side and you are carried back to \
            start again. Five crossings finish the round, and the traffic runs quicker on each. Touch anything on the road and \
            it is over.",
    controls: &["arrow keys walk", "hold them to keep walking"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Hostile, Tag::Navigate, Tag::Timing, Tag::Overhead],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Traffic::new(seed))
}

const LANES: usize = 4;
const CROSSINGS: u32 = 5;
/// How much quicker the traffic runs on each crossing after the first.
const QUICKENING: f32 = 0.2;
const CURB_FAR: f32 = 44.0;
const LANE_TALL: f32 = 54.0;
/// A place to stand between one lane and the next.
const STRIP: f32 = 46.0;
const ROAD_TOP: f32 = CURB_FAR;
const HOME: f32 = 452.0;
const CAR_WIDE: f32 = 74.0;
const CAR_TALL: f32 = 34.0;
const WRAP: f32 = frame::WIDTH as f32 + 200.0;
const PACE: f32 = 210.0;
const WALKER: f32 = 11.0;

struct Lane {
    pace: f32,
    gap: f32,
    offset: f32,
    cars: usize,
}

struct Traffic {
    lanes: Vec<Lane>,
    at: (f32, f32),
    flow: f32,
    crossed: u32,
}

impl Traffic {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let lanes = (0..LANES)
            .map(|lane| {
                let cars = 2 + rng.below(2);
                Lane {
                    pace: rng.range(62.0, 124.0) * if lane % 2 == 0 { 1.0 } else { -1.0 },
                    gap: WRAP / cars as f32,
                    offset: rng.range(0.0, WRAP),
                    cars,
                }
            })
            .collect();
        Traffic { lanes, at: (frame::WIDTH as f32 * 0.5, HOME), flow: 0.0, crossed: 0 }
    }

    fn lane_top(lane: usize) -> f32 {
        ROAD_TOP + lane as f32 * (LANE_TALL + STRIP)
    }

    fn road_bottom() -> f32 {
        Self::lane_top(LANES - 1) + LANE_TALL
    }

    fn pace(&self) -> f32 {
        1.0 + self.crossed as f32 * QUICKENING
    }

    /// Where a car sits along its lane, in traffic that runs quicker each crossing.
    fn car_at(&self, lane: usize, car: usize) -> f32 {
        let line = &self.lanes[lane];
        let along = line.offset + car as f32 * line.gap + line.pace * self.flow;
        along.rem_euclid(WRAP) - 100.0
    }
}

impl Game for Traffic {
    fn update(&mut self, input: &Input) -> Outcome {
        self.flow += self.pace() * TICK;

        if let Some((across, down)) = input.heading() {
            let reach = PACE * TICK;
            self.at.0 = (self.at.0 + across * reach).clamp(WALKER, frame::WIDTH as f32 - WALKER);
            self.at.1 = (self.at.1 + down * reach).clamp(14.0, HOME);
        }

        for lane in 0..LANES {
            let top = Self::lane_top(lane);
            let middle = top + LANE_TALL * 0.5;
            for car in 0..self.lanes[lane].cars {
                let x = self.car_at(lane, car);
                let near_x = self.at.0.clamp(x, x + CAR_WIDE);
                let near_y = self.at.1.clamp(middle - CAR_TALL * 0.5, middle + CAR_TALL * 0.5);
                let (dx, dy) = (self.at.0 - near_x, self.at.1 - near_y);
                if dx * dx + dy * dy < WALKER * WALKER {
                    return Outcome::Lost;
                }
            }
        }

        if self.at.1 <= CURB_FAR - WALKER {
            self.crossed += 1;
            if self.crossed == CROSSINGS {
                return Outcome::Won;
            }
            self.at = (frame::WIDTH as f32 * 0.5, HOME);
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let width = frame::WIDTH as f32;
        let curb = palette::GOOD.mix(palette::BACKDROP, 0.62);
        frame.rect(0.0, 0.0, width, CURB_FAR, curb);
        let road = Self::road_bottom();
        frame.rect(0.0, ROAD_TOP, width, road - ROAD_TOP, palette::PANEL);
        frame.rect(0.0, road, width, frame::HEIGHT as f32 - road, curb);
        for lane in 1..LANES {
            frame.rect(0.0, Self::lane_top(lane) - STRIP, width, STRIP, curb);
        }
        for lane in 0..LANES {
            frame.rect(0.0, Self::lane_top(lane) - 1.0, width, 2.0, palette::EDGE);
            frame.rect(0.0, Self::lane_top(lane) + LANE_TALL - 1.0, width, 2.0, palette::EDGE);
        }
        for lane in 0..LANES {
            let middle = Self::lane_top(lane) + LANE_TALL * 0.5;
            for car in 0..self.lanes[lane].cars {
                let x = self.car_at(lane, car);
                let tint = if lane % 2 == 0 { palette::WARN } else { palette::BAD };
                frame.rect(x, middle - CAR_TALL * 0.5, CAR_WIDE, CAR_TALL, tint);
                let nose = if self.lanes[lane].pace > 0.0 { x + CAR_WIDE - 10.0 } else { x + 2.0 };
                frame.rect(
                    nose,
                    middle - CAR_TALL * 0.5 + 6.0,
                    8.0,
                    CAR_TALL - 12.0,
                    palette::BACKDROP,
                );
            }
        }

        frame.disc(self.at.0, self.at.1, WALKER, palette::PLAYER);
    }
}
