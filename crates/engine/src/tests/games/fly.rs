use super::{GAP, STEP, gaps_of};

#[test]
fn every_course_lays_out_its_pillars_in_order() {
    for seed in 0..40u64 {
        for pair in gaps_of(seed).windows(2) {
            assert!(pair[0].0 < pair[1].0, "seed {seed} laid two pillars out of order");
        }
    }
}

/// The pillars are far enough apart that the craft can climb from one gap to
/// the next. What the generator guarantees is only that a gap moves by at most
/// `STEP`; whether `STEP` is reachable is a relation between the tapping climb,
/// the scroll, and the spacing, and is what a tuning change would break.
#[test]
fn a_tapping_climb_covers_the_rise_between_two_gaps() {
    let gaps = gaps_of(0);
    let spread = gaps[1].0 - gaps[0].0;
    // The tightest pair is two jitters closer together than the spacing.
    let soonest = (spread - 2.0 * 64.0) / super::SCROLL;
    // Tapping holds the craft at about half the lift it gets on a tap.
    let climb = super::LIFT * 0.5;
    assert!(
        climb * soonest > STEP,
        "a climb of {} covers only {} of the {STEP} a gap may move",
        climb,
        climb * soonest
    );
}

/// The band the gaps are drawn from has to have something in it: `clamp`
/// panics when its low bound is above its high one, which is what retuning
/// `GAP`, `ROOF`, or `FLOOR` too far apart would do. Asked of the constants
/// themselves, so it is answered when the crate is built.
#[test]
fn there_is_room_between_the_roof_and_the_floor_to_put_a_gap() {
    const {
        assert!(super::ROOF + GAP * 0.5 + 26.0 < super::FLOOR - GAP * 0.5 - 26.0);
        assert!(GAP > super::CRAFT * 4.0);
    }
}
