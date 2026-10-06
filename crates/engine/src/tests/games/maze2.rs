use super::{
    CELL, COLUMNS, EXIT, ORIGIN, ROOMS, ROWS, cell_of, choke_point, dig, links_of, neighbors,
};
use crate::board::Board;
use crate::maze::Walls;
use crate::rng::Rng;

#[test]
fn every_room_has_two_ways_out_of_it() {
    for seed in 0..64u64 {
        let links = links_of(&dig(&mut Rng::new(seed)));
        assert_eq!(choke_point(&links), None, "seed {seed} left a room that could be cut off");
        assert!(links.iter().all(|ways| ways.len() >= 2), "seed {seed} left a dead end");
    }
}

#[test]
fn a_maze_is_still_a_maze_rather_than_an_empty_room() {
    let every: usize = (0..ROOMS).map(|room| neighbors(room).count()).sum();
    for seed in 0..16u64 {
        let links = links_of(&dig(&mut Rng::new(seed)));
        let ways: usize = links.iter().map(Vec::len).sum();
        assert!(ways < every * 2 / 3, "seed {seed} took down {ways} of {every} walls");
    }
}

/// The path the hound follows is read off the cell grid rather than the room graph.
/// The two agree because rooms sit on the odd cells with exactly one corridor
/// cell between each linked pair, so a room path of `k` is a cell path of `2k`.
#[test]
fn counting_rooms_over_the_cells_agrees_with_counting_them_over_the_rooms() {
    use std::collections::VecDeque;
    for seed in 0..32u64 {
        let walls = dig(&mut Rng::new(seed));
        let links = links_of(&walls);
        let mut rooms = vec![u32::MAX; ROOMS];
        rooms[0] = 0;
        let mut queue = VecDeque::from([0usize]);
        while let Some(room) = queue.pop_front() {
            for &next in &links[room] {
                if rooms[next] == u32::MAX {
                    rooms[next] = rooms[room] + 1;
                    queue.push_back(next);
                }
            }
        }
        let (sx, sy) = cell_of(0);
        let (ex, ey) = cell_of(EXIT);
        let grid =
            Walls { solid: walls, board: Board::at(ORIGIN.0, ORIGIN.1, COLUMNS, ROWS, CELL) };
        let cells = grid.steps((sx, sy));
        assert_eq!(cells[ey * COLUMNS + ex] / 2, rooms[EXIT], "seed {seed}");
    }
}
