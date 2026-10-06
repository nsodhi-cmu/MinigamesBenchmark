//! Find three keys in a dark maze, seeing only as far as the walls allow.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::maze::Walls;
use crate::palette;
use crate::rng::Rng;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "maze3",
    title: "Murky Maze",
    brief: "This maze is unlit, and you see only what you have a clear line to: all around \
            you, as far as the walls, and not a step past them. Whatever is around a corner \
            stays hidden until you go and look. Three keys are somewhere in here. Walk over \
            all three to get out. Nothing in here will hurt you; the only thing against you \
            is that you cannot see where you have not been.",
    controls: &["arrow keys walk", "hold them to keep walking"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Friendly, Tag::Navigate, Tag::Explore, Tag::Overhead, Tag::Memory],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Maze3::new(seed))
}

const COLUMNS: usize = 13;
const ROWS: usize = 9;
const CELL: f32 = 48.0;
const ORIGIN: (f32, f32) = (8.0, 24.0);
const BODY: f32 = 11.0;
const PACE: f32 = 190.0;
const KEYS: usize = 3;
/// Far enough to cross the whole maze, so that walls are the only thing that
/// ever stops sight.
const REACH: f32 = 760.0;
/// How much of the light is lost over that whole reach.
const FADE: f32 = 0.6;
/// The side of a square of picture shaded in one go.
const GRAIN: f32 = 4.0;
const FLOOR: Rgb = Rgb(80, 74, 92);
const WALL: Rgb = Rgb(116, 102, 84);

struct Maze3 {
    walls: Walls<COLUMNS, ROWS>,
    at: (f32, f32),
    keys: Vec<(f32, f32)>,
    found: usize,
}

fn rooms() -> impl Iterator<Item = (usize, usize)> {
    (0..ROWS)
        .flat_map(|row| (0..COLUMNS).map(move |column| (column, row)))
        .filter(|(column, row)| column % 2 == 1 && row % 2 == 1)
}

impl Maze3 {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let walls = Walls::carved(&mut rng, ORIGIN, CELL);
        let mut spots: Vec<(usize, usize)> =
            rooms().filter(|(c, r)| !walls.solid[*r][*c] && (*c, *r) != (1, 1)).collect();
        rng.shuffle(&mut spots);
        let keys = spots.iter().take(KEYS).map(|(c, r)| walls.board.middle(*c, *r)).collect();
        let at = walls.board.middle(1, 1);
        Maze3 { walls, at, keys, found: 0 }
    }

    /// Whether the straight line from where you stand to this spot runs clear.
    /// The first wall it meets is itself in sight; nothing behind that wall is.
    fn in_sight(&self, x: f32, y: f32) -> bool {
        let (dx, dy) = (x - self.at.0, y - self.at.1);
        let far = libm::hypotf(dx, dy);
        if far < 1.0 {
            return true;
        }
        let (cos, sin) = (dx / far, dy / far);
        let (mut column, mut row) = self.walls.board.cell_at((self.at.0, self.at.1));
        let (goal_column, goal_row) = self.walls.board.cell_at((x, y));
        let (step_column, step_row) = (cos.signum() as i32, sin.signum() as i32);
        // How far along the line the next edge of a cell lies, and how far apart
        // such edges are, in each of the two directions.
        let mut edge = (
            edge_at(self.at.0, ORIGIN.0, column, step_column, cos),
            edge_at(self.at.1, ORIGIN.1, row, step_row, sin),
        );
        let apart = (CELL / cos.abs(), CELL / sin.abs());
        // A line can cross no more cells than the grid is wide plus tall.
        for _ in 0..COLUMNS + ROWS {
            if column == goal_column && row == goal_row {
                return true;
            }
            if edge.0 < edge.1 {
                edge.0 += apart.0;
                column += step_column;
            } else {
                edge.1 += apart.1;
                row += step_row;
            }
            if column < 0 || row < 0 || column >= COLUMNS as i32 || row >= ROWS as i32 {
                return false;
            }
            if column == goal_column && row == goal_row {
                return true;
            }
            if self.walls.solid[row as usize][column as usize] {
                return false;
            }
        }
        false
    }

    /// How brightly a spot shows: nothing at all out of sight, and dimmer the
    /// farther off it is.
    fn lit(&self, x: f32, y: f32) -> f32 {
        let far = libm::hypotf(x - self.at.0, y - self.at.1);
        if far > REACH || !self.in_sight(x, y) {
            return 0.0;
        }
        1.0 - FADE * far / REACH
    }
}

/// Where the next cell edge past `at` sits along a line running at `way`, as a
/// distance along that line. Infinite when the line never crosses one.
fn edge_at(at: f32, origin: f32, cell: i32, step: i32, way: f32) -> f32 {
    if way.abs() < 1e-6 {
        return f32::INFINITY;
    }
    let edge = origin + (cell + (step + 1) / 2) as f32 * CELL;
    ((edge - at) / way).max(0.0)
}

impl Game for Maze3 {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some(step) = input.heading() {
            self.at =
                walk::walked(self.at, step, PACE * TICK, |x, y| self.walls.clear((x, y), BODY));
        }

        let mut index = 0;
        while index < self.keys.len() {
            let (kx, ky) = self.keys[index];
            if libm::hypotf(self.at.0 - kx, self.at.1 - ky) < BODY + 10.0 {
                self.keys.swap_remove(index);
                self.found += 1;
            } else {
                index += 1;
            }
        }
        if self.found == KEYS { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        // The floor is shaded a square of picture at a time, so that a shadow
        // edge falls where the corner casting it does. A wall is not: half a
        // wall square lit and half of it dark is a shape nobody would read as
        // a wall, so a wall square is drawn whole as soon as any floor beside
        // it has been seen.
        let mut seen = [[false; COLUMNS]; ROWS];
        let mut y = ORIGIN.1;
        while y < ORIGIN.1 + ROWS as f32 * CELL {
            let mut x = ORIGIN.0;
            while x < ORIGIN.0 + COLUMNS as f32 * CELL {
                let (mx, my) = (x + GRAIN * 0.5, y + GRAIN * 0.5);
                if !self.walls.at(mx, my) {
                    let glow = self.lit(mx, my);
                    if glow > 0.02 {
                        frame.rect(x, y, GRAIN, GRAIN, palette::BACKDROP.mix(FLOOR, glow));
                        let (column, row) = self.walls.board.cell_at((mx, my));
                        seen[row as usize][column as usize] = true;
                    }
                }
                x += GRAIN;
            }
            y += GRAIN;
        }

        for row in 0..ROWS {
            for column in 0..COLUMNS {
                if !self.walls.solid[row][column] {
                    continue;
                }
                let beside = (-1i32..=1).flat_map(|dr| (-1i32..=1).map(move |dc| (dc, dr))).any(
                    |(dc, dr)| {
                        let (c, r) = (column as i32 + dc, row as i32 + dr);
                        c >= 0
                            && r >= 0
                            && (c as usize) < COLUMNS
                            && (r as usize) < ROWS
                            && seen[r as usize][c as usize]
                    },
                );
                if !beside {
                    continue;
                }
                let (x, y) = (ORIGIN.0 + column as f32 * CELL, ORIGIN.1 + row as f32 * CELL);
                let far = libm::hypotf(x + CELL * 0.5 - self.at.0, y + CELL * 0.5 - self.at.1);
                let glow = (1.0 - FADE * far / REACH).clamp(0.15, 1.0);
                frame.rect(x, y, CELL, CELL, palette::BACKDROP.mix(WALL, glow));
            }
        }

        for (kx, ky) in &self.keys {
            let glow = self.lit(*kx, *ky);
            if glow > 0.02 {
                let tint = palette::BACKDROP.mix(palette::WARN, glow.max(0.5));
                frame.disc(*kx, *ky, 7.0, tint);
                frame.rect(*kx + 4.0, *ky - 2.0, 9.0, 4.0, tint);
            }
        }
        frame.disc(self.at.0, self.at.1, BODY, palette::PLAYER);

        let middle = frame::WIDTH as f32 * 0.5;
        frame.pips(
            middle,
            frame::HEIGHT as f32 - 16.0,
            KEYS as u32,
            self.found as u32,
            palette::WARN,
        );
    }
}
