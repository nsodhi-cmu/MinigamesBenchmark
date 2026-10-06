use super::{BOARD, MARGIN, MARK, Targets};

/// Placing the marks one to a cell of a coarse grid keeps any two of them
/// apart by construction, for every seed there is.
#[test]
fn no_two_marks_ever_start_on_top_of_one_another() {
    for seed in 0..256u64 {
        let marks = Targets::new(seed).marks;
        for (slot, mark) in marks.iter().enumerate() {
            for other in &marks[..slot] {
                let apart = libm::hypotf(mark.at.0 - other.at.0, mark.at.1 - other.at.1);
                assert!(apart >= MARK * 4.0, "seed {seed} put two marks {apart} apart");
            }
        }
    }
}

#[test]
fn every_mark_starts_on_the_board() {
    for seed in 0..256u64 {
        for mark in Targets::new(seed).marks {
            assert!(
                (BOARD.0 + MARGIN..=BOARD.2 - MARGIN).contains(&mark.at.0)
                    && (BOARD.1 + MARGIN..=BOARD.3 - MARGIN).contains(&mark.at.1),
                "seed {seed} put a mark at {:?}, off the board",
                mark.at
            );
        }
    }
}
