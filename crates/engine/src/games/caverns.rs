//! Swim through flooded caverns, taking air where the caverns hold any.

use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::{Input, Key};
use crate::map::Map;
use crate::palette;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "caverns",
    title: "Underwater Caverns",
    brief: "Three flooded caverns, each entered at the top left with the way out above the \
            water at the top right. You float at the surface on your own. Hold DOWN to sink, \
            and once under, tap UP for a kick upward: let go and the water carries you back up \
            slowly. LEFT and RIGHT swim. Where there is air you walk and jump instead. The bar \
            at the top left is your breath: underwater it runs down, and in air, whether at the \
            surface or in a pocket of it caught under the rock, it fills again quickly. Run out \
            of breath and the round is over. The way out is never straight across.",
    controls: &[
        "LEFT and RIGHT swim or walk",
        "tap UP or SPACE to kick upward, or to jump in air",
        "hold DOWN to sink",
    ],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Hostile, Tag::Navigate, Tag::Timing, Tag::Physics],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Caverns::new())
}

pub(crate) const COLUMNS: usize = 32;
pub(crate) const ROWS: usize = 24;
pub(crate) const CELL: f32 = 20.0;
pub(crate) const TOP: f32 = 0.0;
/// Half the side of the square you are.
pub(crate) const HALF: f32 = 7.0;
const SWIM: f32 = 112.0;
/// The lift the water gives, the most it carries you up at, and the kick a tap up gives.
const FLOAT: f32 = 300.0;
const BOB: f32 = 54.0;
const KICK: f32 = 190.0;
/// What holding down adds, and the fastest it takes you down.
const PADDLE: f32 = 620.0;
const DIVE: f32 = 150.0;
const RUN: f32 = 150.0;
const GRAVITY: f32 = 1400.0;
const JUMP: f32 = 430.0;
const FALL: f32 = 520.0;
/// Ticks of breath held under water, and ticks of air it takes to fill again.
pub(crate) const BREATH: f32 = 6.0 * TICK_HZ as f32;
const REFILL: f32 = 1.1 * TICK_HZ as f32;

const ROCK: Rgb = Rgb(74, 62, 54);
const ROCK_EDGE: Rgb = Rgb(104, 88, 76);
const WATER: Rgb = Rgb(30, 96, 190);
const DEEP: Rgb = Rgb(20, 66, 140);
const DIVER: Rgb = Rgb(250, 214, 96);

/// Each cavern a row at a time: `#` rock, `.` water, `a` air, `S` the way in,
/// and `E` the way out.
pub(crate) const CAVERNS: [[&str; ROWS]; 3] = [
    [
        "################################",
        "################################",
        "##Saaaa##################aaaEE##",
        "##.....##########.............##",
        "##.....##########.............##",
        "###..############..#############",
        "###..############..#############",
        "###..############..#############",
        "###...########aa#..#############",
        "###...#####........#############",
        "###...#####........#############",
        "###...#####..###################",
        "###...#####..###################",
        "###...#####..###################",
        "###...#####..###################",
        "####..#####..###################",
        "####..##aa#..###################",
        "####.........###################",
        "####.........###################",
        "################################",
        "################################",
        "################################",
        "################################",
        "################################",
    ],
    [
        "################################",
        "################################",
        "##Saaaa##################aaaEE##",
        "##.....#######................##",
        "##.....#######................##",
        "###..#########..################",
        "###..#########..################",
        "###..#########..################",
        "###..#########..#aa#############",
        "###..#####..........############",
        "###..#####..........############",
        "###..#####..####################",
        "###..#####..####################",
        "###..#####..####################",
        "###..#####..####################",
        "###..#####..####################",
        "###..#####..####################",
        "###..##aa#..####################",
        "###.........####################",
        "###.........####################",
        "################################",
        "################################",
        "################################",
        "################################",
    ],
    [
        "################################",
        "################################",
        "##Saaaa##################aaaEE##",
        "##.....##################.....##",
        "##.....##################.....##",
        "###..######################..###",
        "###..######################..###",
        "###..#######aa#############..###",
        "###..####........#######aa#..###",
        "###..####........####........###",
        "###..####..####..####........###",
        "###..####..####..####..####..###",
        "###..####..####..####..####..###",
        "###..####..####..####..####..###",
        "###..#aa#..####..####..####..###",
        "###........####..####..####..###",
        "###........####..####..#########",
        "###############..####..#########",
        "###############..#aa#..#########",
        "###############........#########",
        "###############........#########",
        "################################",
        "################################",
        "################################",
    ],
];

/// A cavern as a map of what is where. Off the map is rock: the way out is a
/// mark on it, not the edge of it.
#[derive(Clone, Copy)]
pub(crate) struct Cavern {
    pub(crate) map: Map<'static>,
}

impl Cavern {
    pub(crate) fn of(rows: &'static [&'static str]) -> Cavern {
        Cavern { map: Map::new(rows, COLUMNS, CELL) }
    }

    fn rock(self, column: i32, row: i32) -> bool {
        self.map.mark(column, row).unwrap_or(b'#') == b'#'
    }

    pub(crate) fn blocked(self, at: (f32, f32)) -> bool {
        self.map.touches(at, HALF, |column, row| self.rock(column, row))
    }

    /// Where the diver's head is, which is what decides whether they can
    /// breathe and whether they have reached the way out.
    fn head(at: (f32, f32)) -> (f32, f32) {
        (at.0, at.1 - HALF * 0.6)
    }

    /// Whether the diver's head is out of the water, which is what lets them
    /// breathe and what stops the water carrying them up any farther.
    pub(crate) fn breathable(self, at: (f32, f32)) -> bool {
        self.map.under(Self::head(at)).unwrap_or(b'#') != b'.'
    }

    /// Whether the diver is in the water, which is what decides how they move.
    pub(crate) fn wet(self, at: (f32, f32)) -> bool {
        self.map.under(at).unwrap_or(b'#') == b'.'
    }

    pub(crate) fn find(self, mark: u8) -> (f32, f32) {
        self.map.find(mark)
    }

    /// Whether the diver is out: the way out is above the water, so it is
    /// reached by surfacing in it rather than by swimming under it.
    pub(crate) fn out(self, at: (f32, f32)) -> bool {
        self.map.under(Self::head(at)).unwrap_or(b'#') == b'E'
    }
}

/// Where the diver is, how fast they are sinking or falling, and how much
/// breath is left, from one down to nothing.
#[derive(Clone, Copy)]
pub(crate) struct Diver {
    pub(crate) at: (f32, f32),
    pub(crate) fall: f32,
    pub(crate) breath: f32,
}

/// What a tick of swimming or walking comes to.
pub(crate) enum Step {
    Moved(Diver),
    Out,
    Drowned,
}

impl Diver {
    /// One tick, with `across` one way or the other, `up` set while a tap up is
    /// being made, and `down` while down is held.
    pub(crate) fn step(self, cavern: Cavern, across: f32, up: bool, down: bool) -> Step {
        let (mut x, mut y) = self.at;
        let mut fall = self.fall;

        let (dx, dy) = if cavern.wet(self.at) {
            if up {
                fall = -KICK;
            } else if down {
                fall = (fall + PADDLE * TICK).min(DIVE);
            } else {
                fall = (fall - FLOAT * TICK).max(-BOB);
            }
            // Floating: the lift of the water stops carrying once the head is out of it.
            if fall < 0.0 && cavern.breathable((x, y)) {
                fall = 0.0;
            }
            (across * SWIM * TICK, fall * TICK)
        } else {
            let standing = cavern.blocked((x, y + 1.0));
            if standing && up {
                fall = -JUMP;
            }
            fall = (fall + GRAVITY * TICK).min(FALL);
            (across * RUN * TICK, fall * TICK)
        };

        let was = (x, y);
        x = walk::slid(x, dx, HALF, CELL, |x| cavern.blocked((x, y)));
        y = walk::slid(y, dy, HALF, CELL, |y| cavern.blocked((x, y)));
        if y != was.1 + dy {
            fall = 0.0;
        }

        let breath = if cavern.breathable((x, y)) {
            (self.breath + 1.0 / REFILL).min(1.0)
        } else {
            self.breath - 1.0 / BREATH
        };
        if cavern.out((x, y)) {
            Step::Out
        } else if breath <= 0.0 {
            Step::Drowned
        } else {
            Step::Moved(Diver { at: (x, y), fall, breath })
        }
    }
}

pub(crate) struct Caverns {
    pub(crate) cavern: usize,
    diver: Diver,
}

impl Caverns {
    pub(crate) fn new() -> Self {
        Caverns::entering(0)
    }

    pub(crate) fn entering(cavern: usize) -> Self {
        let map = Cavern::of(&CAVERNS[cavern]);
        Caverns { cavern, diver: Diver { at: map.find(b'S'), fall: 0.0, breath: 1.0 } }
    }
}

impl Game for Caverns {
    fn update(&mut self, input: &Input) -> Outcome {
        let cavern = Cavern::of(&CAVERNS[self.cavern]);
        let across = input.axis(Key::Left, Key::Right);
        let up = input.lift_pressed();
        match self.diver.step(cavern, across, up, input.held(Key::Down)) {
            Step::Moved(diver) => {
                self.diver = diver;
                Outcome::Playing
            }
            Step::Out if self.cavern + 1 == CAVERNS.len() => Outcome::Won,
            Step::Out => {
                *self = Caverns::entering(self.cavern + 1);
                Outcome::Playing
            }
            Step::Drowned => Outcome::Lost,
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let cavern = Cavern::of(&CAVERNS[self.cavern]);
        for row in 0..ROWS {
            for column in 0..COLUMNS {
                let (x, y) = (column as f32 * CELL, TOP + row as f32 * CELL);
                match cavern.map.mark(column as i32, row as i32).unwrap_or(b'#') {
                    b'#' => {
                        frame.bordered(x, y, CELL, CELL, 1.0, ROCK, ROCK_EDGE);
                    }
                    b'E' => frame.rect(x, y, CELL, CELL, palette::GOOD),
                    b'.' => {
                        frame.rect(x, y, CELL, CELL, WATER);
                        frame.rect(x + 3.0, y + 3.0, 3.0, 3.0, DEEP);
                    }
                    _ => frame.rect(x, y, CELL, CELL, palette::BACKDROP),
                }
            }
        }

        let (x, y) = (self.diver.at.0, TOP + self.diver.at.1);
        frame.rect(x - HALF, y - HALF, HALF * 2.0, HALF * 2.0, DIVER);
        frame.rect(x - 3.0, y - 4.0, 6.0, 4.0, palette::BACKDROP);

        let bar = 96.0;
        frame.rect(12.0, TOP + 6.0, bar + 4.0, 12.0, palette::BACKDROP);
        let left = self.diver.breath * bar;
        let tint = if self.diver.breath < 0.3 { palette::BAD } else { palette::PLAYER };
        frame.rect(14.0, TOP + 8.0, left, 8.0, tint);
    }
}

#[cfg(test)]
#[path = "../tests/games/caverns.rs"]
mod tests;
