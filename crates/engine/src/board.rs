//! A board of square cells laid out on the screen.

use crate::frame;

/// Cells of one size, side by side with no gap between them. Where a game
/// draws a gap it draws it inside a cell, so that every point on the board
/// belongs to exactly one of them.
#[derive(Clone, Copy)]
pub struct Board {
    pub left: f32,
    pub top: f32,
    pub cell: f32,
    pub columns: usize,
    pub rows: usize,
}

impl Board {
    pub const fn at(left: f32, top: f32, columns: usize, rows: usize, cell: f32) -> Board {
        Board { left, top, cell, columns, rows }
    }

    /// Centered on the screen.
    pub const fn centered(columns: usize, rows: usize, cell: f32) -> Board {
        Board {
            left: (frame::WIDTH as f32 - cell * columns as f32) * 0.5,
            top: (frame::HEIGHT as f32 - cell * rows as f32) * 0.5,
            cell,
            columns,
            rows,
        }
    }

    pub fn corner(&self, column: usize, row: usize) -> (f32, f32) {
        (self.left + column as f32 * self.cell, self.top + row as f32 * self.cell)
    }

    pub fn middle(&self, column: usize, row: usize) -> (f32, f32) {
        (self.left + (column as f32 + 0.5) * self.cell, self.top + (row as f32 + 0.5) * self.cell)
    }

    /// The cell a point is in, which may be off the board either way.
    pub fn cell_at(&self, at: (f32, f32)) -> (i32, i32) {
        (
            ((at.0 - self.left) / self.cell).floor() as i32,
            ((at.1 - self.top) / self.cell).floor() as i32,
        )
    }

    /// The cell a far edge stops in. Half open: an edge lying exactly on a
    /// boundary belongs to the cell behind it, not the one past it, so a body
    /// standing flush against a wall is not standing in it.
    pub fn cell_before(&self, at: (f32, f32)) -> (i32, i32) {
        (
            ((at.0 - self.left) / self.cell).ceil() as i32 - 1,
            ((at.1 - self.top) / self.cell).ceil() as i32 - 1,
        )
    }

    /// Whether a cell is on the board.
    pub fn holds(&self, column: i32, row: i32) -> bool {
        (0..self.columns as i32).contains(&column) && (0..self.rows as i32).contains(&row)
    }

    /// The cell a point is on, across then down, if it is on the board at all.
    pub fn under(&self, at: (f32, f32)) -> Option<(usize, usize)> {
        let (column, row) = self.cell_at(at);
        self.holds(column, row).then_some((column as usize, row as usize))
    }

    /// The cell a point is on, a point off the board counting as the nearest
    /// cell on it. What a game asks where a body is kept on the board anyway.
    pub fn nearest(&self, at: (f32, f32)) -> (usize, usize) {
        let (column, row) = self.cell_at(at);
        (
            column.clamp(0, self.columns as i32 - 1) as usize,
            row.clamp(0, self.rows as i32 - 1) as usize,
        )
    }

    /// How far across and down the whole board reaches.
    pub fn span(&self) -> (f32, f32) {
        (self.columns as f32 * self.cell, self.rows as f32 * self.cell)
    }
}

#[cfg(test)]
#[path = "tests/board.rs"]
mod tests;
