//! A grid of walls: digging one, standing in one, and drawing one.

use crate::board::Board;
use crate::frame::Frame;
use crate::palette;
use crate::rng::Rng;

/// A grid of square walls on a board, `solid[row][column]`.
pub struct Walls<const COLUMNS: usize, const ROWS: usize> {
    pub solid: [[bool; COLUMNS]; ROWS],
    pub board: Board,
}

impl<const COLUMNS: usize, const ROWS: usize> Walls<COLUMNS, ROWS> {
    /// A perfect maze on the odd cells: one route between any two of them, dug
    /// by walking from the top left and turning back at dead ends.
    pub fn carved(rng: &mut Rng, origin: (f32, f32), cell: f32) -> Self {
        let board = Board::at(origin.0, origin.1, COLUMNS, ROWS, cell);
        Walls { solid: carve(rng), board }
    }

    /// Whether a point is inside a wall. Anywhere off the grid is walled.
    pub fn at(&self, x: f32, y: f32) -> bool {
        let (column, row) = self.board.cell_at((x, y));
        !self.board.holds(column, row) || self.solid[row as usize][column as usize]
    }

    /// Whether a body `half` across may stand with its middle at `at`, which
    /// asks about its four corners.
    pub fn clear(&self, at: (f32, f32), half: f32) -> bool {
        !([(-half, -half), (half, -half), (-half, half), (half, half)])
            .into_iter()
            .any(|(dx, dy)| self.at(at.0 + dx, at.1 + dy))
    }

    /// How many cells every open cell is from `from`, walking the grid. A cell
    /// the walls cut off is `u32::MAX` away, and so is a wall itself.
    pub fn steps(&self, from: (usize, usize)) -> Vec<u32> {
        let mut steps = vec![u32::MAX; COLUMNS * ROWS];
        steps[from.1 * COLUMNS + from.0] = 0;
        let mut queue = std::collections::VecDeque::from([from]);
        while let Some((column, row)) = queue.pop_front() {
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = ((column as i32 + dx) as usize, (row as i32 + dy) as usize);
                if nx < COLUMNS
                    && ny < ROWS
                    && !self.solid[ny][nx]
                    && steps[ny * COLUMNS + nx] == u32::MAX
                {
                    steps[ny * COLUMNS + nx] = steps[row * COLUMNS + column] + 1;
                    queue.push_back((nx, ny));
                }
            }
        }
        steps
    }

    /// The walls as raised panels, over the floor behind them.
    pub fn draw(&self, frame: &mut Frame) {
        let (across, down) = self.board.span();
        let cell = self.board.cell;
        frame.rect(self.board.left, self.board.top, across, down, palette::BACKDROP);
        for (row, line) in self.solid.iter().enumerate() {
            for (column, wall) in line.iter().enumerate() {
                if *wall {
                    let (x, y) = self.board.corner(column, row);
                    frame.rect(x, y, cell, cell, palette::PANEL);
                    frame.rect(x, y, cell, 2.0, palette::EDGE);
                }
            }
        }
    }
}

fn carve<const COLUMNS: usize, const ROWS: usize>(rng: &mut Rng) -> [[bool; COLUMNS]; ROWS] {
    let mut walls = [[true; COLUMNS]; ROWS];
    let mut stack = vec![(1usize, 1usize)];
    walls[1][1] = false;
    while let Some(&(column, row)) = stack.last() {
        let mut ways = Vec::new();
        for (dx, dy) in [(2i32, 0i32), (-2, 0), (0, 2), (0, -2)] {
            let (nx, ny) = (column as i32 + dx, row as i32 + dy);
            if nx > 0
                && ny > 0
                && nx < COLUMNS as i32 - 1
                && ny < ROWS as i32 - 1
                && walls[ny as usize][nx as usize]
            {
                ways.push((nx as usize, ny as usize));
            }
        }
        if ways.is_empty() {
            stack.pop();
            continue;
        }
        let (nx, ny) = ways[rng.below(ways.len())];
        walls[(ny + row) / 2][(nx + column) / 2] = false;
        walls[ny][nx] = false;
        stack.push((nx, ny));
    }
    walls
}

#[cfg(test)]
#[path = "tests/maze.rs"]
mod tests;
