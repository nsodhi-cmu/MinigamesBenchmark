use super::{CELLS, lay};
use crate::rng::Rng;

fn pairs() -> impl Iterator<Item = (usize, usize)> {
    (0..CELLS).flat_map(|a| (a + 1..CELLS).map(move |b| (a, b)))
}

/// `lay` draws its answer from the pairs whose total no other pair makes, and
/// takes for granted that there is one to draw. There always is: on a board of
/// nine numbers that all differ, the two smallest add to less than any other
/// pair can. Dealing many boards holds both halves — drawing from an empty set
/// would panic, and the two smallest are checked against every other pair.
#[test]
fn the_two_smallest_numbers_on_a_board_always_make_a_total_no_other_pair_makes() {
    let mut rng = Rng::new(7);
    for _ in 0..500 {
        let (board, total, (a, b)) = lay(&mut rng);
        assert_eq!(board[a] + board[b], total, "the answer does not make the total");
        let mut order = board;
        order.sort_unstable();
        let smallest = order[0] + order[1];
        let makes = pairs().filter(|(i, j)| board[*i] + board[*j] == smallest).count();
        assert_eq!(makes, 1, "{makes} pairs make {smallest} on {board:?}");
    }
}
