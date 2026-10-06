//! A place written out as rows of letters, one letter to a cell.
//!
//! The map's own pixels start at its top left corner; where on the screen that
//! corner sits is the game's business, not the map's.

use crate::board::Board;

/// Rows of letters of equal length, `rows[row]` running across.
#[derive(Clone, Copy)]
pub struct Map<'a> {
    pub rows: &'a [&'static str],
    pub board: Board,
}

impl<'a> Map<'a> {
    pub fn new(rows: &'a [&'static str], columns: usize, cell: f32) -> Map<'a> {
        Map { rows, board: Board::at(0.0, 0.0, columns, rows.len(), cell) }
    }

    /// The letter at a cell, or nothing where the cell is off the map. What
    /// being off the map means is the game's to say: for one game a body falls
    /// out of the bottom, for another the rock goes on forever.
    pub fn mark(&self, column: i32, row: i32) -> Option<u8> {
        self.board.holds(column, row).then(|| self.rows[row as usize].as_bytes()[column as usize])
    }

    /// The letter under a point.
    pub fn under(&self, at: (f32, f32)) -> Option<u8> {
        let (column, row) = self.board.cell_at(at);
        self.mark(column, row)
    }

    /// The middle of the first cell carrying `mark`, row by row.
    pub fn find(&self, mark: u8) -> (f32, f32) {
        let at = self
            .rows
            .iter()
            .enumerate()
            .find_map(|(row, line)| line.bytes().position(|b| b == mark).map(|c| (c, row)))
            .expect("the mark is on the map");
        self.board.middle(at.0, at.1)
    }

    /// Every cell a square of half side `half` centered on `at` reaches into.
    pub fn spanned(&self, at: (f32, f32), half: f32) -> impl Iterator<Item = (i32, i32)> {
        let low = self.board.cell_at((at.0 - half, at.1 - half));
        let high = self.board.cell_before((at.0 + half, at.1 + half));
        (low.0..=high.0).flat_map(move |column| (low.1..=high.1).map(move |row| (column, row)))
    }

    /// Whether any cell it reaches into is one `solid` calls solid.
    pub fn touches(&self, at: (f32, f32), half: f32, solid: impl Fn(i32, i32) -> bool) -> bool {
        self.spanned(at, half).any(|(column, row)| solid(column, row))
    }
}

#[cfg(test)]
#[path = "tests/map.rs"]
mod tests;
