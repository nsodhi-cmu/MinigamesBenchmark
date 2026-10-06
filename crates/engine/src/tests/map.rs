use super::Map;

const ROWS: [&str; 4] = ["####", "#S.#", "#.E#", "####"];

fn map() -> Map<'static> {
    Map::new(&ROWS, 4, 10.0)
}

#[test]
fn a_cell_off_the_map_carries_no_mark_rather_than_a_made_up_one() {
    let map = map();
    assert_eq!(map.mark(1, 1), Some(b'S'));
    assert_eq!(map.mark(-1, 1), None);
    assert_eq!(map.mark(4, 1), None);
    assert_eq!(map.mark(1, -1), None);
    assert_eq!(map.mark(1, 4), None);
}

#[test]
fn a_mark_is_found_at_the_middle_of_its_own_cell() {
    assert_eq!(map().find(b'E'), (25.0, 25.0));
    assert_eq!(map().under((25.0, 25.0)), Some(b'E'));
}

/// A body wider than a cell has to be stopped by any cell it reaches into, not
/// only by the one its middle is in.
#[test]
fn a_body_reaches_into_every_cell_it_overlaps() {
    let map = map();
    let span: Vec<(i32, i32)> = map.spanned((15.0, 15.0), 1.0).collect();
    assert_eq!(span, vec![(1, 1)]);
    assert!(map.touches((15.0, 15.0), 6.0, |c, r| map.mark(c, r) == Some(b'#')));
    assert!(!map.touches((15.0, 15.0), 1.0, |c, r| map.mark(c, r) == Some(b'#')));
}

/// A walk into a wall ends flush against it, leaving no gap that the next turn
/// would have to jump across, and no overlap either.
#[test]
fn a_step_into_a_wall_stops_against_it_rather_than_short_of_it() {
    // Cells are ten across and a wall stands from twenty on; the body is three
    // across from its middle, so it may stand with its middle up to seventeen.
    let wall = |along: f32| along + 3.0 >= 20.0;
    let stopped = crate::walk::slid(15.0, 6.0, 3.0, 10.0, wall);
    assert!(stopped < 17.0, "the body at {stopped} is standing in the wall");
    assert!(stopped > 16.9, "the body stopped at {stopped}, short of the wall at 17");
    assert_eq!(crate::walk::slid(15.0, 1.0, 3.0, 10.0, |_| false), 16.0, "a clear step is whole");
}
