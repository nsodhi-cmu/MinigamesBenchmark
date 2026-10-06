//! Two squares on two sets of keys, parted by gates that only the other can open.

use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Input, Key};
use crate::map::Map;
use crate::palette;
use crate::walk::Hopper;

pub const SPEC: Spec = Spec {
    name: "duo",
    title: "Dungeon Duo",
    brief: "Two squares are parted in a dungeon: a blue one and a red one. A gate is a \
            stack of colored bars in a wall, and it is open only while someone stands on \
            the floor tile with the neon top of the same color, which is always on the \
            other side of it. It shuts the moment the tile is left, and a square inside it \
            when it shuts is crushed and the round is over. Bring the two squares together \
            so that they touch, in three rooms.",
    controls: &[
        "A and D run the blue square, W jumps",
        "LEFT and RIGHT run the red square, UP jumps",
    ],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Puzzle, Tag::Navigate, Tag::Platformer],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Duo::new())
}

pub(crate) const COLUMNS: usize = 32;
pub(crate) const ROWS: usize = 24;
pub(crate) const CELL: f32 = 20.0;
const TOP: f32 = 0.0;
pub(crate) const HALF: f32 = 7.0;

const ROCK_EDGE: Rgb = Rgb(98, 100, 114);
const BLUE: Rgb = Rgb(72, 140, 255);
const RED: Rgb = Rgb(236, 72, 80);
/// The color of each gate and its plate, in the order of `GATES`.
pub(crate) const NEON: [Rgb; 3] = [Rgb(72, 255, 120), Rgb(255, 80, 232), Rgb(255, 190, 60)];

/// The gates: each plate letter and the gate letter it opens.
pub(crate) const GATES: [(u8, u8); 3] = [(b'p', b'P'), (b'q', b'Q'), (b'r', b'R')];

/// Each room a row at a time from the top: `#` rock, `1` where blue starts, `2`
/// where red starts, `p`, `q`, and `r` plates, and `P`, `Q`, and `R` the gates
/// they raise.
pub(crate) const ROOMS: [[&str; ROWS]; 3] = [
    [
        "################################",
        "################################",
        "#..............##.........######",
        "#..............##.........######",
        "#..............##.........######",
        "#..............##.........######",
        "#..............##.........######",
        "#.pp###........##qq#......######",
        "#..............#####......######",
        "#..............#####......######",
        "#........###...##....###..P....#",
        "#..............##....###..P....#",
        "#..............##....###..P....#",
        "#....###.......##..##########..#",
        "#..............##..########....#",
        "#..............##..............#",
        "#.........###..##............###",
        "#..............##..............#",
        "#..............##..............#",
        "#....###.......QQ..........##..#",
        "#..............QQ..............#",
        "#1.............QQ.2............#",
        "################################",
        "################################",
    ],
    [
        "################################",
        "################################",
        "#.........##........##.........#",
        "#.........##........##.........#",
        "#.........##........##.........#",
        "#.........PP........##.........#",
        "#.........PP........##.........#",
        "#......###PP........##pp#......#",
        "#.........##........##.........#",
        "#.........##........##.........#",
        "#.###.....##........##.....###.#",
        "#.........##........##.........#",
        "#.........##........##.........#",
        "#......#####........#####......#",
        "#.........##........##.........#",
        "#.........##........##.........#",
        "#####.....##........##.....#####",
        "#.........##........##.........#",
        "#.........##........##.........#",
        "#.....###.##........QQ.###.....#",
        "#.........##........QQ.........#",
        "#1........##.qq.....QQ........2#",
        "################################",
        "################################",
    ],
    [
        "################################",
        "################################",
        "#..............##..............#",
        "#..............##..............#",
        "#..............##..............#",
        "#..............##..............#",
        "#..............QQ..............#",
        "#..............QQ..............#",
        "#..............QQ.2.........pp.#",
        "#..............#################",
        "#...############################",
        "#...###########RR..............#",
        "#..............RR..............#",
        "#..............RR..............#",
        "############rr###qq##..........#",
        "#####################..........#",
        "###########....##.....###......#",
        "###########....##.....###......#",
        "###########....##.....###......#",
        "###########....PP..........###.#",
        "###########....PP..........###.#",
        "###########.1..PP..........###.#",
        "################################",
        "################################",
    ],
];

/// A room as a map: which cells are solid while the gates stand as they do.
#[derive(Clone, Copy)]
pub(crate) struct Room {
    pub(crate) map: Map<'static>,
}

impl Room {
    pub(crate) fn of(rows: &'static [&'static str]) -> Room {
        Room { map: Map::new(rows, COLUMNS, CELL) }
    }

    pub(crate) fn mark(self, column: i32, row: i32) -> u8 {
        self.map.mark(column, row).unwrap_or(b'#')
    }

    /// Whether a square of half side `HALF` centered on `at` overlaps rock, a
    /// plate, or a shut gate.
    pub(crate) fn blocked(self, at: (f32, f32), open: [bool; 3]) -> bool {
        self.map.touches(at, HALF, |column, row| match self.mark(column, row) {
            b'#' | b'p' | b'q' | b'r' => true,
            mark => GATES.iter().zip(open).any(|((_, gate), open)| mark == *gate && !open),
        })
    }

    fn touching(self, at: (f32, f32), letter: u8) -> bool {
        self.map.touches(at, HALF, |column, row| self.mark(column, row) == letter)
    }
}

pub(crate) struct Duo {
    pub(crate) room: usize,
    pub(crate) blue: Hopper,
    pub(crate) red: Hopper,
}

impl Duo {
    pub(crate) fn new() -> Self {
        Duo::entering(0)
    }

    pub(crate) fn entering(room: usize) -> Self {
        let map = Room::of(&ROOMS[room]).map;
        let body = |mark| Hopper { at: map.find(mark), climb: 0.0 };
        Duo { room, blue: body(b'1'), red: body(b'2') }
    }

    pub(crate) fn together(&self) -> bool {
        (self.blue.at.0 - self.red.at.0).abs() < HALF * 2.0
            && (self.blue.at.1 - self.red.at.1).abs() < HALF * 2.0
    }

    /// Which gates are open: those with a square standing on their plate.
    pub(crate) fn open(&self) -> [bool; 3] {
        let room = Room::of(&ROOMS[self.room]);
        GATES.map(|(plate, _)| {
            [self.blue, self.red].iter().any(|b| room.touching((b.at.0, b.at.1 + 1.0), plate))
        })
    }

    /// Whether a square is inside a gate that is shut.
    fn crushed(&self, open: [bool; 3]) -> bool {
        let room = Room::of(&ROOMS[self.room]);
        GATES.iter().zip(open).any(|((_, gate), open)| {
            !open && [self.blue, self.red].iter().any(|b| room.touching(b.at, *gate))
        })
    }
}

impl Game for Duo {
    fn update(&mut self, input: &Input) -> Outcome {
        let open = self.open();
        if self.crushed(open) {
            return Outcome::Lost;
        }
        let room = Room::of(&ROOMS[self.room]);
        let blocked = |at| room.blocked(at, open);
        let step = |body: Hopper, left, right, up| {
            body.stepped(input.axis(left, right), input.held(up), HALF, CELL, blocked)
        };
        self.blue = step(self.blue, Key::A, Key::D, Key::W);
        self.red = step(self.red, Key::Left, Key::Right, Key::Up);
        if !self.together() {
            return Outcome::Playing;
        }
        if self.room + 1 == ROOMS.len() {
            return Outcome::Won;
        }
        *self = Duo::entering(self.room + 1);
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let room = Room::of(&ROOMS[self.room]);
        let open = self.open();
        for row in 0..ROWS {
            for column in 0..COLUMNS {
                let (x, y) = (column as f32 * CELL, TOP + row as f32 * CELL);
                let mark = room.mark(column as i32, row as i32);
                if let Some(which) = GATES.iter().position(|(_, gate)| *gate == mark) {
                    if open[which] {
                        frame.rect(x, y, CELL, CELL, NEON[which].mix(palette::BACKDROP, 0.85));
                        continue;
                    }
                    frame.rect(x, y, CELL, CELL, NEON[which].mix(palette::BACKDROP, 0.7));
                    for bar in [4.0, 12.0] {
                        frame.rect(x + bar, y, 4.0, CELL, NEON[which]);
                    }
                    continue;
                }
                let plate = GATES.iter().position(|(plate, _)| *plate == mark);
                if mark != b'#' && plate.is_none() {
                    continue;
                }
                frame.bordered(x, y, CELL, CELL, 1.0, palette::EDGE, ROCK_EDGE);
                if let Some(which) = plate {
                    frame.rect(x, y, CELL, 5.0, NEON[which]);
                }
            }
        }
        for (body, tint) in [(self.blue, BLUE), (self.red, RED)] {
            let (x, y) = (body.at.0, TOP + body.at.1);
            frame.rect(x - HALF, y - HALF, HALF * 2.0, HALF * 2.0, tint);
            frame.rect(x - 4.0, y - 3.0, 2.0, 3.0, palette::BACKDROP);
            frame.rect(x + 2.0, y - 3.0, 2.0, 3.0, palette::BACKDROP);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/duo.rs"]
mod tests;
