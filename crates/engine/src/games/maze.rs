//! Find the way out of a fresh maze.

use crate::frame::Frame;
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::maze::Walls;
use crate::palette;
use crate::rng::Rng;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "maze",
    title: "Mythic Maze",
    brief: "You are the blue dot in the top left of a maze. The way out is the green square \
            in the bottom right. Walk there. Walls stop you but do not hurt you.",
    controls: &["arrow keys walk", "hold them to keep walking"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Friendly, Tag::Navigate, Tag::Overhead],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Warren::new(seed))
}

const COLUMNS: usize = 21;
const ROWS: usize = 15;
const CELL: f32 = 28.0;
const ORIGIN: (f32, f32) = (26.0, 30.0);
const RADIUS: f32 = 8.0;
const PACE: f32 = 230.0;

struct Warren {
    walls: Walls<COLUMNS, ROWS>,
    at: (f32, f32),
}

impl Warren {
    fn new(seed: u64) -> Self {
        let walls = Walls::carved(&mut Rng::new(seed), ORIGIN, CELL);
        let at = walls.board.middle(1, 1);
        Warren { walls, at }
    }
}

impl Game for Warren {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some(step) = input.heading() {
            self.at =
                walk::walked(self.at, step, PACE * TICK, |x, y| self.walls.clear((x, y), RADIUS));
        }
        let exit = self.walls.board.middle(COLUMNS - 2, ROWS - 2);
        let (dx, dy) = (self.at.0 - exit.0, self.at.1 - exit.1);
        if dx * dx + dy * dy < CELL * CELL * 0.16 { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        self.walls.draw(frame);
        let (ex, ey) = self.walls.board.middle(COLUMNS - 2, ROWS - 2);
        frame.rect(ex - CELL * 0.35, ey - CELL * 0.35, CELL * 0.7, CELL * 0.7, palette::GOOD);
        frame.disc(self.at.0, self.at.1, RADIUS, palette::PLAYER);
    }
}
