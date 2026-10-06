use super::Board;

const BOARD: Board = Board::centered(6, 6, 64.0);

/// Every point on the board belongs to one cell, and the cell it belongs to is
/// the one drawn there: no point falls between two of them.
#[test]
fn a_point_on_the_board_is_on_the_cell_that_is_drawn_there() {
    for column in 0..BOARD.columns {
        for row in 0..BOARD.rows {
            let middle = BOARD.middle(column, row);
            assert_eq!(BOARD.under(middle), Some((column, row)));
            let (x, y) = BOARD.corner(column, row);
            assert_eq!(BOARD.under((x + 0.5, y + 0.5)), Some((column, row)));
            assert_eq!(
                BOARD.under((x + BOARD.cell - 0.5, y + BOARD.cell - 0.5)),
                Some((column, row))
            );
        }
    }
}

#[test]
fn a_point_off_the_board_is_on_no_cell_at_all() {
    let (across, down) = BOARD.span();
    assert_eq!(BOARD.under((BOARD.left - 0.5, BOARD.top + 1.0)), None);
    assert_eq!(BOARD.under((BOARD.left + 1.0, BOARD.top - 0.5)), None);
    assert_eq!(BOARD.under((BOARD.left + across + 0.5, BOARD.top + 1.0)), None);
    assert_eq!(BOARD.under((BOARD.left + 1.0, BOARD.top + down + 0.5)), None);
}

/// Where `under` gives up, `nearest` folds the point onto the rim, and
/// everywhere `under` answers the two agree.
#[test]
fn a_point_off_the_board_is_nearest_the_cell_on_the_rim_beside_it() {
    let (across, down) = BOARD.span();
    let last = (BOARD.columns - 1, BOARD.rows - 1);
    assert_eq!(BOARD.nearest((BOARD.left - 500.0, BOARD.top - 500.0)), (0, 0));
    assert_eq!(BOARD.nearest((BOARD.left + across + 500.0, BOARD.top + down + 500.0)), last);
    assert_eq!(BOARD.nearest((BOARD.left - 500.0, BOARD.top + down + 500.0)), (0, last.1));
    for column in 0..BOARD.columns {
        for row in 0..BOARD.rows {
            assert_eq!(BOARD.nearest(BOARD.middle(column, row)), (column, row));
        }
    }
}
