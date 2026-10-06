use super::Walls;
use crate::rng::Rng;

const COLUMNS: usize = 21;
const ROWS: usize = 15;

/// A perfect maze is a tree over its open cells: every one reachable, and as
/// many ways between them as there are cells bar one. Either property alone
/// would pass for a grid with a loop in it or for one cut in two.
#[test]
fn a_carved_maze_has_exactly_one_route_between_any_two_of_its_cells() {
    for seed in 0..32u64 {
        let walls: Walls<COLUMNS, ROWS> = Walls::carved(&mut Rng::new(seed), (0.0, 0.0), 10.0);
        let open: Vec<(usize, usize)> = (0..ROWS)
            .flat_map(|row| (0..COLUMNS).map(move |column| (column, row)))
            .filter(|(column, row)| !walls.solid[*row][*column])
            .collect();

        let ways = open
            .iter()
            .flat_map(|(column, row)| {
                [(1usize, 0usize), (0, 1)].into_iter().map(move |(dx, dy)| (column + dx, row + dy))
            })
            .filter(|(column, row)| *column < COLUMNS && *row < ROWS && !walls.solid[*row][*column])
            .count();
        assert_eq!(ways, open.len() - 1, "seed {seed} carved a loop or a second maze");

        let mut seen = vec![open[0]];
        let mut queue = vec![open[0]];
        while let Some((column, row)) = queue.pop() {
            for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (column as i32 + dx, row as i32 + dy);
                if nx < 0 || ny < 0 || nx >= COLUMNS as i32 || ny >= ROWS as i32 {
                    continue;
                }
                let next = (nx as usize, ny as usize);
                if !walls.solid[next.1][next.0] && !seen.contains(&next) {
                    seen.push(next);
                    queue.push(next);
                }
            }
        }
        assert_eq!(seen.len(), open.len(), "seed {seed} walled part of the maze off");
    }
}

#[test]
fn anywhere_off_the_grid_is_walled_so_a_body_cannot_walk_out_of_it() {
    let walls: Walls<COLUMNS, ROWS> = Walls::carved(&mut Rng::new(1), (10.0, 20.0), 8.0);
    assert!(walls.at(0.0, 0.0), "the place above and left of the grid is open");
    assert!(walls.at(10.0 + COLUMNS as f32 * 8.0, 40.0), "the place past the grid is open");
    let middle = walls.board.middle(1, 1);
    assert!(walls.clear(middle, 1.0), "the cell the digging starts in is walled");
}
