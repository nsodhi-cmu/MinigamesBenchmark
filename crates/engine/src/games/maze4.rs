//! Cross a maze far wider than the screen, which scrolls around a player who never moves off the middle.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::maze::Walls;
use crate::palette;
use crate::rng::Rng;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "maze4",
    title: "Mega Maze",
    brief: "This maze is many screens wide and you see only the part of it you are in. You \
            are the blue dot, and you never leave the middle of the screen: the maze slides \
            around you as you walk, so which way you came is only what you remember. Walls \
            stop you but do not hurt you, and nothing in here is hidden or out to get you. \
            You start in the middle of the maze, and the green square is at the point of it \
            furthest from there by way of the corridors, so it is a long walk whichever way \
            you go.",
    controls: &["arrow keys walk", "hold them to keep walking"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Friendly, Tag::Navigate, Tag::Memory, Tag::Overhead],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Maze4::new(seed))
}

pub(crate) const COLUMNS: usize = 33;
pub(crate) const ROWS: usize = 25;
pub(crate) const CELL: f32 = 38.0;
const BODY: f32 = 11.0;
/// How fast the player eases back to the middle of the corridor they walk along.
const SNAP: f32 = 70.0;
pub(crate) const PACE: f32 = 340.0;
/// The middle of the screen, where the player always is.
const MIDDLE: (f32, f32) = (frame::WIDTH as f32 * 0.5, frame::HEIGHT as f32 * 0.5);

const WALL_EDGE: Rgb = Rgb(92, 102, 138);

pub(crate) struct Maze4 {
    pub(crate) walls: Walls<COLUMNS, ROWS>,
    pub(crate) at: (f32, f32),
    pub(crate) exit: (f32, f32),
}

/// The cell the player starts in, in the middle of the maze.
pub(crate) const START: (usize, usize) = ((COLUMNS / 2) | 1, (ROWS / 2) | 1);
/// How far each cell is from the one the player starts in, walking the maze.
pub(crate) fn spread(walls: &Walls<COLUMNS, ROWS>) -> Vec<u32> {
    walls.steps(START)
}

/// The cell the exit is in: the one furthest from the start by way of the maze.
pub(crate) fn goal(walls: &Walls<COLUMNS, ROWS>) -> (usize, usize) {
    let far = spread(walls);
    let cell = (0..COLUMNS * ROWS)
        .filter(|cell| far[*cell] != u32::MAX)
        .max_by_key(|cell| far[*cell])
        .expect("the maze has somewhere to walk");
    (cell % COLUMNS, cell / COLUMNS)
}

/// The maze, dug from the top left corner.
pub(crate) fn dig(rng: &mut Rng) -> Walls<COLUMNS, ROWS> {
    Walls::carved(rng, (0.0, 0.0), CELL)
}

/// Slid toward the middle of the cell it is in, which is what keeps a walk
/// along a corridor lined up with the turnings off it.
fn middled(along: f32) -> f32 {
    walk::eased(along, ((along / CELL).floor() + 0.5) * CELL, SNAP * TICK)
}

/// How far ahead of its middle the player is felt for on a step: as far as
/// the edge of the cell, so that a wall stops them in the middle of the last
/// cell before it rather than up against it.
fn ahead(step: f32) -> f32 {
    if step == 0.0 { 0.0 } else { step.signum() * (CELL * 0.5 - BODY) }
}

impl Maze4 {
    pub(crate) fn new(seed: u64) -> Self {
        let walls = dig(&mut Rng::new(seed));
        let (column, row) = goal(&walls);
        let (at, exit) = (walls.board.middle(START.0, START.1), walls.board.middle(column, row));
        Maze4 { walls, at, exit }
    }

    pub(crate) fn blocked(&self, x: f32, y: f32) -> bool {
        !self.walls.clear((x, y), BODY)
    }
}

impl Game for Maze4 {
    fn update(&mut self, input: &Input) -> Outcome {
        let (across, down) = input.arrows();
        if across != 0.0 && down == 0.0 {
            self.at.1 = middled(self.at.1);
        } else if down != 0.0 && across == 0.0 {
            self.at.0 = middled(self.at.0);
        }
        if let Some((across, down)) = input.heading() {
            let (dx, dy) = (across * PACE * TICK, down * PACE * TICK);
            let (x, y) = self.at;
            let x = walk::slid(x, dx, CELL * 0.5, CELL, |x| self.blocked(x + ahead(dx), y));
            let y = walk::slid(y, dy, CELL * 0.5, CELL, |y| self.blocked(x, y + ahead(dy)));
            self.at = (x, y);
        }
        let (dx, dy) = (self.at.0 - self.exit.0, self.at.1 - self.exit.1);
        if dx * dx + dy * dy < CELL * CELL * 0.16 { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        let corner = (MIDDLE.0 - self.at.0, MIDDLE.1 - self.at.1);
        frame.rect(0.0, 0.0, frame::WIDTH as f32, frame::HEIGHT as f32, palette::BACKDROP);
        for row in 0..ROWS {
            for column in 0..COLUMNS {
                if !self.walls.solid[row][column] {
                    continue;
                }
                let (x, y) = (corner.0 + column as f32 * CELL, corner.1 + row as f32 * CELL);
                frame.bordered(x, y, CELL, CELL, 1.0, palette::EDGE, WALL_EDGE);
            }
        }
        let (ex, ey) = self.exit;
        frame.rect(
            corner.0 + ex - CELL * 0.4,
            corner.1 + ey - CELL * 0.4,
            CELL * 0.8,
            CELL * 0.8,
            palette::GOOD,
        );
        frame.disc(MIDDLE.0, MIDDLE.1, BODY, palette::PLAYER);
    }
}

#[cfg(test)]
#[path = "../tests/games/maze4.rs"]
mod tests;
