//! Fly a jetpack through caves past spinning bars of fire.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::map::Map;
use crate::palette;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "pyro",
    title: "Pyro Passage",
    brief: "Fly a jetpack through three caves to the green exit in each. You fall unless you \
            hold UP or SPACE, which fires the jetpack and lifts you, and LEFT and RIGHT move you \
            sideways. \
            Bars of fire turn steadily about the blocks they are fixed to. Touching a wall does no \
            harm, but touching fire ends the round. The way to the exit is not always straight \
            there, so find the gaps in the walls.",
    controls: &["hold UP or SPACE to fire the jetpack", "LEFT and RIGHT move sideways"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Hostile, Tag::Navigate, Tag::Timing, Tag::Physics],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Pyro::new())
}

pub(crate) const COLUMNS: usize = 32;
pub(crate) const ROWS: usize = 24;
pub(crate) const CELL: f32 = 20.0;
const TOP: f32 = 0.0;
/// Half the side of the square you are.
pub(crate) const HALF: f32 = 7.0;
const SIDEWAYS: f32 = 120.0;
const GRAVITY: f32 = 540.0;
const THRUST: f32 = 1200.0;
const RISE: f32 = 150.0;
const FALL: f32 = 180.0;
/// Every bar comes back to where it started after this many ticks.
pub(crate) const PERIOD: u32 = 480;
const SPACING: f32 = 11.0;
const FLAME: f32 = 5.0;

const ROCK: Rgb = Rgb(66, 58, 64);
const ROCK_EDGE: Rgb = Rgb(92, 82, 88);
const PIVOT: Rgb = Rgb(128, 96, 64);
const FIRE: Rgb = Rgb(255, 84, 24);
const EXHAUST: Rgb = Rgb(196, 226, 255);
const PACK: Rgb = Rgb(150, 156, 168);

/// A bar of fire fixed to the block in a column and row: how many flames long,
/// how many whole turns it makes each period, clockwise when positive, and the
/// angle it starts at in degrees.
pub(crate) struct Bar {
    pub(crate) block: (i32, i32),
    pub(crate) flames: usize,
    pub(crate) turns: i32,
    pub(crate) start: f32,
}

/// `#` rock, `S` where you start, and `E` the exit.
pub(crate) struct Cave {
    pub(crate) map: [&'static str; ROWS],
    pub(crate) bars: &'static [Bar],
}

pub(crate) const CAVES: [Cave; 3] = [
    Cave {
        map: [
            "################################",
            "################################",
            "#..............................#",
            "#.............................E#",
            "#.............................E#",
            "#.............................E#",
            "#.............................E#",
            "#..............................#",
            "########...#####################",
            "#..............................#",
            "#..............................#",
            "#..............................#",
            "#..............................#",
            "#..............................#",
            "#..............................#",
            "#################...############",
            "#..............................#",
            "#..............................#",
            "#..............................#",
            "#..............................#",
            "#......S.......................#",
            "#..............................#",
            "################################",
            "################################",
        ],
        bars: &[
            Bar { block: (11, 18), flames: 6, turns: 2, start: 0.0 },
            Bar { block: (16, 15), flames: 7, turns: 2, start: 180.0 },
            Bar { block: (11, 8), flames: 7, turns: -2, start: 0.0 },
            Bar { block: (22, 4), flames: 6, turns: 2, start: 90.0 },
        ],
    },
    Cave {
        map: [
            "################################",
            "################################",
            "#...........###................#",
            "#...........###EE..............#",
            "#..S........###EE..............#",
            "#...........###................#",
            "#...........##.................#",
            "#...........##.................#",
            "#...........##.................#",
            "#...........###############....#",
            "#...........##.................#",
            "#...........##.................#",
            "#...........##.................#",
            "#...........##.................#",
            "#...........##...###############",
            "#...........##.................#",
            "#...........##.................#",
            "#...........##.................#",
            "#..............................#",
            "#..............................#",
            "#..............................#",
            "#..............................#",
            "################################",
            "################################",
        ],
        bars: &[
            Bar { block: (6, 10), flames: 5, turns: -3, start: 225.0 },
            Bar { block: (12, 20), flames: 5, turns: 3, start: 270.0 },
            Bar { block: (17, 14), flames: 7, turns: -2, start: 90.0 },
            Bar { block: (26, 9), flames: 8, turns: 2, start: 45.0 },
            Bar { block: (21, 5), flames: 5, turns: 3, start: 270.0 },
        ],
    },
    Cave {
        map: [
            "################################",
            "################################",
            "#.......##.....................#",
            "#EE.....##.....................#",
            "#EE.....##.....................#",
            "#.......##.....................#",
            "#.......##......##.............#",
            "#.......##......##.............#",
            "#.......##......##.............#",
            "#.......##......##.............#",
            "#...............##.............#",
            "#...............##.............#",
            "#...............##.............#",
            "#...............##.............#",
            "##########################.....#",
            "#..............................#",
            "#..............................#",
            "#.......##.........##..........#",
            "#..................##..........#",
            "#..................##..........#",
            "#..S...............##..........#",
            "#..................##..........#",
            "################################",
            "################################",
        ],
        bars: &[
            Bar { block: (12, 21), flames: 6, turns: 3, start: 180.0 },
            Bar { block: (20, 16), flames: 5, turns: -2, start: 270.0 },
            Bar { block: (25, 14), flames: 5, turns: 2, start: 90.0 },
            Bar { block: (17, 6), flames: 5, turns: 3, start: 0.0 },
            Bar { block: (8, 9), flames: 5, turns: -3, start: 180.0 },
            Bar { block: (4, 7), flames: 4, turns: 2, start: 0.0 },
        ],
    },
];

impl Cave {
    /// Off the map is rock: a cave is closed and the exit is a mark on it.
    pub(crate) fn grid(&self) -> Map<'_> {
        Map::new(&self.map, COLUMNS, CELL)
    }

    /// A bar's own block is solid too, so a bar always turns about something.
    fn solid(&self, column: i32, row: i32) -> bool {
        self.grid().mark(column, row).unwrap_or(b'#') == b'#'
            || self.bars.iter().any(|bar| bar.block == (column, row))
    }

    /// Whether a square of half side `HALF` centered on `at` overlaps rock.
    fn blocked(&self, at: (f32, f32)) -> bool {
        self.grid().touches(at, HALF, |column, row| self.solid(column, row))
    }

    /// The middle of the first cell marked `mark`, row by row.
    pub(crate) fn find(&self, mark: u8) -> (f32, f32) {
        self.grid().find(mark)
    }

    /// The middle of every flame of every bar there is `tick` ticks into the cave.
    fn flames(&self, tick: u32) -> impl Iterator<Item = (f32, f32)> + '_ {
        let phase = (tick % PERIOD) as f32 / PERIOD as f32;
        self.bars.iter().flat_map(move |bar| {
            let (px, py) = ((bar.block.0 as f32 + 0.5) * CELL, (bar.block.1 as f32 + 0.5) * CELL);
            let angle = (bar.start + 360.0 * bar.turns as f32 * phase).to_radians();
            let (sin, cos) = libm::sincosf(angle);
            (0..bar.flames).map(move |flame| {
                let reach = SPACING * flame as f32;
                (px + reach * cos, py + reach * sin)
            })
        })
    }

    pub(crate) fn burned(&self, at: (f32, f32), tick: u32) -> bool {
        self.flames(tick).any(|(fx, fy)| {
            let dx = fx - fx.clamp(at.0 - HALF, at.0 + HALF);
            let dy = fy - fy.clamp(at.1 - HALF, at.1 + HALF);
            dx * dx + dy * dy < FLAME * FLAME
        })
    }

    pub(crate) fn out(&self, at: (f32, f32)) -> bool {
        self.grid().under(at) == Some(b'E')
    }
}

/// Where you are on the cave's map and how fast you are rising or falling.
#[derive(Clone, Copy)]
pub(crate) struct Flier {
    pub(crate) at: (f32, f32),
    pub(crate) climb: f32,
}

impl Flier {
    /// One tick of flying, `across` one way or the other or not at all.
    pub(crate) fn flown(self, cave: &Cave, across: f32, up: bool) -> Flier {
        let pull = if up { GRAVITY - THRUST } else { GRAVITY };
        let mut climb = (self.climb + pull * TICK).clamp(-RISE, FALL);
        let (mut x, mut y) = self.at;
        let dx = across * SIDEWAYS * TICK;
        x = walk::slid(x, dx, HALF, CELL, |x| cave.blocked((x, y)));
        let dy = climb * TICK;
        let fell = y + dy;
        y = walk::slid(y, dy, HALF, CELL, |y| cave.blocked((x, y)));
        if y != fell {
            climb = 0.0;
        }
        Flier { at: (x, y), climb }
    }
}

pub(crate) struct Pyro {
    pub(crate) cave: usize,
    tick: u32,
    flier: Flier,
    firing: bool,
}

impl Pyro {
    pub(crate) fn new() -> Self {
        Pyro::entering(0)
    }

    pub(crate) fn entering(cave: usize) -> Self {
        let flier = Flier { at: CAVES[cave].find(b'S'), climb: 0.0 };
        Pyro { cave, tick: 0, flier, firing: false }
    }
}

impl Game for Pyro {
    fn update(&mut self, input: &Input) -> Outcome {
        let cave = &CAVES[self.cave];
        self.tick += 1;
        self.firing = input.lift_held();
        let across = input.axis(Key::Left, Key::Right);
        self.flier = self.flier.flown(cave, across, self.firing);
        if cave.out(self.flier.at) {
            if self.cave + 1 == CAVES.len() {
                return Outcome::Won;
            }
            *self = Pyro::entering(self.cave + 1);
            return Outcome::Playing;
        }
        if cave.burned(self.flier.at, self.tick) { Outcome::Lost } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let cave = &CAVES[self.cave];
        frame.rect(0.0, TOP, frame::WIDTH as f32, ROWS as f32 * CELL, palette::BACKDROP);
        for row in 0..ROWS {
            for column in 0..COLUMNS {
                let (x, y) = (column as f32 * CELL, TOP + row as f32 * CELL);
                match cave.grid().mark(column as i32, row as i32).unwrap_or(b'#') {
                    b'#' => {
                        frame.bordered(x, y, CELL, CELL, 1.0, ROCK, ROCK_EDGE);
                    }
                    b'E' => frame.rect(x, y, CELL, CELL, palette::GOOD),
                    _ => {}
                }
            }
        }

        for bar in cave.bars {
            let (x, y) = (bar.block.0 as f32 * CELL, TOP + bar.block.1 as f32 * CELL);
            frame.rect(x + 1.0, y + 1.0, CELL - 2.0, CELL - 2.0, PIVOT);
        }

        let (x, y) = (self.flier.at.0, TOP + self.flier.at.1);
        if self.firing {
            frame.poly(&[[x - 4.0, y + HALF], [x + 4.0, y + HALF], [x, y + HALF + 9.0]], EXHAUST);
        }
        frame.rect(x - HALF, y - HALF + 3.0, HALF * 2.0, HALF * 2.0 - 3.0, PACK);
        frame.rect(x - 4.0, y - HALF, 8.0, HALF * 2.0 - 1.0, palette::PLAYER);

        for (fx, fy) in cave.flames(self.tick) {
            frame.disc(fx, TOP + fy, FLAME, FIRE);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/pyro.rs"]
mod tests;
