use super::{Cell, LEVELS, TURNS, board_of, closed, sockets_of};

#[test]
fn every_board_has_three_sockets_and_two_terminals() {
    for level in 0..LEVELS {
        let board = board_of(level);
        assert_eq!(sockets_of(&board).len(), 3, "board {level} does not offer three sockets");
        for wanted in [true, false] {
            let count =
                board.iter().flatten().filter(|cell| **cell == Cell::Terminal(wanted)).count();
            assert_eq!(count, 1, "board {level} does not have one terminal on each side");
        }
    }
}

/// Every board must be closable by the three sockets, and must not already be
/// closed, or there would be nothing to do.
#[test]
fn every_board_can_be_closed_and_none_starts_closed() {
    for level in 0..LEVELS {
        let board = board_of(level);
        assert!(!closed(&board), "board {level} is closed before it is touched");

        let sockets = sockets_of(&board);
        let mut answers = 0;
        for first in 0..TURNS.len() {
            for second in 0..TURNS.len() {
                for third in 0..TURNS.len() {
                    let mut trial = board.clone();
                    for (socket, piece) in sockets.iter().zip([first, second, third]) {
                        trial[socket.1][socket.0] = Cell::Socket(piece);
                    }
                    if closed(&trial) {
                        answers += 1;
                    }
                }
            }
        }
        assert!(answers > 0, "board {level} cannot be closed at all");
    }
}
