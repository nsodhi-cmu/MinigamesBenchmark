//! Run and jump through rooms whose red and blue blocks trade places on a beat.

use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Input, Key};
use crate::map::Map;
use crate::palette;
use crate::walk::Hopper;

pub const SPEC: Spec = Spec {
    name: "flip",
    title: "Flip Flop Hop",
    brief: "Run and jump through three rooms to the green door in each. Red blocks and blue \
            blocks take turns being solid: while one color is solid the other is only an \
            outline you pass straight through. Every two seconds the solid color pulses \
            lighter three times, a beat apart, and on the fourth beat the two colors swap at \
            once. Stand on a block that goes, and you fall. Be inside a block when it comes \
            back, or fall off the bottom of the room, and the round is over.",
    controls: &["LEFT and RIGHT run", "hold UP or SPACE to jump whenever you land"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Hostile, Tag::Navigate, Tag::Timing, Tag::Platformer],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Flip::new())
}

pub(crate) const COLUMNS: usize = 32;
pub(crate) const ROWS: usize = 24;
pub(crate) const CELL: f32 = 20.0;
const TOP: f32 = 0.0;
/// Half the side of the square you are.
pub(crate) const HALF: f32 = 7.0;
/// Ticks between beats, and beats between swaps.
pub(crate) const BEAT: u32 = 30;
pub(crate) const PHASE: u32 = BEAT * 4;
const FLASH: u32 = 8;

const ROCK_EDGE: Rgb = Rgb(98, 100, 114);
const RED: Rgb = Rgb(218, 56, 60);
const BLUE: Rgb = Rgb(56, 108, 232);
const HERO: Rgb = Rgb(244, 204, 84);

/// Each room a row at a time from the top: `#` rock, `R` a red block, `B` a
/// blue block, `S` where you start, and `E` the door.
pub(crate) const ROOMS: [[&str; ROWS]; 3] = [
    [
        "################################",
        "################################",
        "#..............................#",
        "#..............................#",
        "#..............................#",
        "#..............................#",
        "#..............................#",
        "#..............................#",
        "#..............................#",
        "#..............................#",
        "#..............................#",
        "#..............................#",
        "#..............................#",
        "#..........................B...#",
        "#..........................B...#",
        "#..........................B...#",
        "#..........................B..E#",
        "#.S........................B..E#",
        "#######RRRRRRBBBBBBRRRRRR#######",
        "#######..................#######",
        "#######..................#######",
        "#######..................#######",
        "#######..................#######",
        "................................",
    ],
    [
        "################################",
        "################################",
        "#.................##############",
        "#.................##############",
        "#.................##############",
        "#.................##############",
        "#.................##############",
        "#.................##############",
        "#.......................B......#",
        "#.......................B.....E#",
        "#.......................B.....E#",
        "#.................##############",
        "#............BBBB..............#",
        "#..............................#",
        "#.........RRRR.................#",
        "#..............................#",
        "#............BBBB..............#",
        "#..............................#",
        "#.S.......RRRR.................#",
        "#########......................#",
        "#########......................#",
        "#########......................#",
        "#########......................#",
        "................................",
    ],
    [
        "################################",
        "################################",
        "#.............##################",
        "#.............##################",
        "#.............##################",
        "#.............##################",
        "#..............RRRRRBBBBB.B..R.#",
        "#.........................B..RE#",
        "#.........................B..RE#",
        "#...........RR.BBBBBRRRRR.######",
        "#..............................#",
        "#.........BB...................#",
        "#..............................#",
        "#.......RR.....................#",
        "#..............................#",
        "#.....BB.......................#",
        "#..............................#",
        "#...RR.........................#",
        "#.S............................#",
        "####...........................#",
        "####...........................#",
        "####...........................#",
        "####...........................#",
        "................................",
    ],
];

pub(crate) fn blue_on(tick: u32) -> bool {
    tick / PHASE % 2 == 1
}

/// A room as a map: which cells are solid at a given tick and where things are.
#[derive(Clone, Copy)]
pub(crate) struct Room {
    pub(crate) map: Map<'static>,
}

impl Room {
    pub(crate) fn of(rows: &'static [&'static str]) -> Room {
        Room { map: Map::new(rows, COLUMNS, CELL) }
    }

    /// Beside a room is wall, but under it is open air: falling out of the
    /// bottom is how a room is lost, so the floor is not the edge of the map.
    pub(crate) fn mark(self, column: i32, row: i32) -> u8 {
        match self.map.mark(column, row) {
            Some(mark) => mark,
            None if (0..COLUMNS as i32).contains(&column) => b'.',
            None => b'#',
        }
    }

    fn solid(self, column: i32, row: i32, tick: u32) -> bool {
        match self.mark(column, row) {
            b'#' => true,
            b'R' => !blue_on(tick),
            b'B' => blue_on(tick),
            _ => false,
        }
    }

    /// Whether a square of half side `HALF` centered on `at` overlaps anything solid.
    pub(crate) fn blocked(self, at: (f32, f32), tick: u32) -> bool {
        self.map.touches(at, HALF, |column, row| self.solid(column, row, tick))
    }

    pub(crate) fn find(self, mark: u8) -> (f32, f32) {
        self.map.find(mark)
    }

    pub(crate) fn out(self, at: (f32, f32)) -> bool {
        self.mark((at.0 / CELL).floor() as i32, (at.1 / CELL).floor() as i32) == b'E'
    }
}

/// What a tick of running and jumping comes to.
pub(crate) enum Step {
    Moved(Hopper),
    Out,
    Crushed,
    Fell,
}

/// One tick at `tick`, running `across` one way or the other or not at all.
pub(crate) fn step(hopper: Hopper, room: Room, tick: u32, across: f32, jump: bool) -> Step {
    if room.blocked(hopper.at, tick) {
        return Step::Crushed;
    }
    let hopper = hopper.stepped(across, jump, HALF, CELL, |at| room.blocked(at, tick));
    if room.out(hopper.at) {
        Step::Out
    } else if hopper.at.1 - HALF > ROWS as f32 * CELL {
        Step::Fell
    } else {
        Step::Moved(hopper)
    }
}

pub(crate) struct Flip {
    pub(crate) room: usize,
    tick: u32,
    hopper: Hopper,
}

impl Flip {
    pub(crate) fn new() -> Self {
        Flip::entering(0)
    }

    fn entering(room: usize) -> Self {
        let hopper = Hopper { at: Room::of(&ROOMS[room]).find(b'S'), climb: 0.0 };
        Flip { room, tick: 0, hopper }
    }
}

impl Game for Flip {
    fn update(&mut self, input: &Input) -> Outcome {
        self.tick += 1;
        let across = input.axis(Key::Left, Key::Right);
        let jump = input.lift_held();
        match step(self.hopper, Room::of(&ROOMS[self.room]), self.tick, across, jump) {
            Step::Moved(hopper) => {
                self.hopper = hopper;
                Outcome::Playing
            }
            Step::Out if self.room + 1 == ROOMS.len() => Outcome::Won,
            Step::Out => {
                *self = Flip::entering(self.room + 1);
                Outcome::Playing
            }
            Step::Crushed | Step::Fell => Outcome::Lost,
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let room = Room::of(&ROOMS[self.room]);
        let into = self.tick % PHASE;
        let pulse = into >= BEAT && into % BEAT < FLASH;
        let blue = blue_on(self.tick);
        for row in 0..ROWS {
            for column in 0..COLUMNS {
                let (x, y) = (column as f32 * CELL, TOP + row as f32 * CELL);
                let (tint, on) = match room.mark(column as i32, row as i32) {
                    b'#' => {
                        frame.bordered(x, y, CELL, CELL, 1.0, palette::EDGE, ROCK_EDGE);
                        continue;
                    }
                    b'E' => {
                        frame.rect(x, y, CELL, CELL, palette::GOOD);
                        continue;
                    }
                    b'R' => (RED, !blue),
                    b'B' => (BLUE, blue),
                    _ => continue,
                };
                if on {
                    let tint = if pulse { tint.mix(palette::INK, 0.5) } else { tint };
                    let edge = tint.mix(palette::BACKDROP, 0.35);
                    frame.bordered(x, y, CELL, CELL, 2.0, tint, edge);
                } else {
                    frame.outline(
                        x + 2.0,
                        y + 2.0,
                        CELL - 4.0,
                        CELL - 4.0,
                        2.0,
                        tint.mix(palette::BACKDROP, 0.4),
                    );
                }
            }
        }
        let (x, y) = (self.hopper.at.0, TOP + self.hopper.at.1);
        frame.rect(x - HALF, y - HALF, HALF * 2.0, HALF * 2.0, HERO);
        frame.rect(x - 4.0, y - 3.0, 2.0, 3.0, palette::BACKDROP);
        frame.rect(x + 2.0, y - 3.0, 2.0, 3.0, palette::BACKDROP);
    }
}

#[cfg(test)]
#[path = "../tests/games/flip.rs"]
mod tests;
