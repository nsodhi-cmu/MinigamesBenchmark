//! Push every crate onto a mark, where a crate can be pushed but never pulled.

use crate::board::Board;
use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Input, Key};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "pallets",
    title: "Push Pallets",
    brief: "An eight by eight warehouse has walls, crates, and marks on the floor. Walk with \
            the arrow keys, one square a press. Walking into a crate pushes it one square, if \
            the square beyond it is empty. Crates cannot be pulled, and two crates cannot be \
            pushed at once. Put a crate on every mark. There are three rooms.",
    controls: &["arrow keys walk one square and push a crate in the way"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle, Tag::Overhead, Tag::Grid],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Pallets::new())
}

pub(crate) const SIDE: usize = 8;
const CELL: f32 = 52.0;
const BOARD: Board = Board::centered(SIDE, SIDE, CELL);

/// `#` a wall, `G` a mark, `B` a crate, `*` a crate on a mark, `P` the player.
pub(crate) const ROOMS: [[&str; SIDE]; 3] = [
    [
        "########", "###P####", "##GB.###", "###.B..#", "###....#", "####..G#", "########",
        "########",
    ],
    [
        "########", "######.G", "#####...", "###..BB.", "#GG.B..#", "####P###", "########",
        "########",
    ],
    [
        "########", "#####...", "#####.B.", "####...#", "##.GBB.#", "##.G.G##", "#####P##",
        "########",
    ],
];

const CRATE: Rgb = Rgb(176, 124, 64);
const GRAIN: Rgb = Rgb(120, 80, 38);

/// The square next to `cell` the way `arrow` points, if the warehouse has one.
pub(crate) fn beside(cell: usize, arrow: Key) -> Option<usize> {
    let (row, column) = (cell / SIDE, cell % SIDE);
    match arrow {
        Key::Up if row > 0 => Some(cell - SIDE),
        Key::Down if row + 1 < SIDE => Some(cell + SIDE),
        Key::Left if column > 0 => Some(cell - 1),
        Key::Right if column + 1 < SIDE => Some(cell + 1),
        _ => None,
    }
}

/// A room as it stands, one bit a square for the walls, the marks, and the crates.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Layout {
    pub(crate) walls: u64,
    pub(crate) marks: u64,
    pub(crate) crates: u64,
    pub(crate) player: usize,
}

impl Layout {
    pub(crate) fn of(room: usize) -> Layout {
        let mut layout = Layout { walls: 0, marks: 0, crates: 0, player: 0 };
        for (cell, letter) in ROOMS[room].concat().chars().enumerate() {
            let bit = 1u64 << cell;
            match letter {
                '#' => layout.walls |= bit,
                'G' => layout.marks |= bit,
                'B' => layout.crates |= bit,
                '*' => {
                    layout.marks |= bit;
                    layout.crates |= bit;
                }
                'P' => layout.player = cell,
                _ => {}
            }
        }
        layout
    }

    fn open(&self, cell: usize) -> bool {
        (self.walls | self.crates) >> cell & 1 == 0
    }

    /// The layout after one press of `arrow`, which is the same layout if nothing moves.
    pub(crate) fn walked(&self, arrow: Key) -> Layout {
        let mut next = *self;
        let Some(to) = beside(self.player, arrow) else { return next };
        if self.crates >> to & 1 == 1 {
            let Some(beyond) = beside(to, arrow).filter(|beyond| self.open(*beyond)) else {
                return next;
            };
            next.crates ^= 1 << to | 1 << beyond;
        } else if self.walls >> to & 1 == 1 {
            return next;
        }
        next.player = to;
        next
    }
}

pub(crate) struct Pallets {
    pub(crate) room: usize,
    pub(crate) layout: Layout,
}

impl Pallets {
    pub(crate) fn new() -> Self {
        Pallets { room: 0, layout: Layout::of(0) }
    }
}

impl Game for Pallets {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some((arrow, _)) = input.arrow() {
            self.layout = self.layout.walked(arrow);
        }
        if self.layout.crates != self.layout.marks {
            return Outcome::Playing;
        }
        if self.room + 1 == ROOMS.len() {
            return Outcome::Won;
        }
        self.room += 1;
        self.layout = Layout::of(self.room);
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let (across, down) = BOARD.span();
        frame.rect(BOARD.left - 6.0, BOARD.top - 6.0, across + 12.0, down + 12.0, palette::EDGE);
        let layout = &self.layout;
        for cell in 0..SIDE * SIDE {
            let (x, y) = BOARD.corner(cell % SIDE, cell / SIDE);
            let bit = |mask: u64| mask >> cell & 1 == 1;
            if bit(layout.walls) {
                frame.rect(x, y, CELL, CELL, palette::EDGE);
                continue;
            }
            frame.bordered(x, y, CELL, CELL, 1.0, palette::PANEL, palette::BACKDROP);
            if bit(layout.crates) {
                let tint = if bit(layout.marks) { palette::GOOD } else { CRATE };
                frame.bordered(x + 6.0, y + 6.0, CELL - 12.0, CELL - 12.0, 3.0, tint, GRAIN);
                frame.cross(x + CELL * 0.5, y + CELL * 0.5, CELL * 0.5 - 9.0, 3.0, GRAIN);
            }
            if bit(layout.marks) {
                frame.outline(x + 15.0, y + 15.0, CELL - 30.0, CELL - 30.0, 3.0, palette::WARN);
            }
            if cell == layout.player {
                frame.disc(x + CELL * 0.5, y + CELL * 0.5, 16.0, palette::PLAYER);
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/pallets.rs"]
mod tests;
