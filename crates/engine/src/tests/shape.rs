use super::stroke_hits;

const BOX: (f32, f32) = (300.0, 200.0);
const HALF: (f32, f32) = (17.0, 17.0);

#[test]
fn a_stroke_straight_through_the_middle_hits_it() {
    assert!(stroke_hits((0.0, 200.0), (640.0, 200.0), BOX, HALF));
    assert!(stroke_hits((300.0, 0.0), (300.0, 480.0), BOX, HALF));
    assert!(stroke_hits((0.0, 0.0), (600.0, 400.0), BOX, HALF));
}

#[test]
fn a_stroke_that_goes_by_does_not() {
    assert!(!stroke_hits((0.0, 260.0), (640.0, 260.0), BOX, HALF));
    assert!(!stroke_hits((0.0, 0.0), (640.0, 0.0), BOX, HALF));
}

/// The stretch of a stroke is what counts, not where it ends: a stroke that
/// stops short of the box misses it even though the line it lies on would not.
#[test]
fn a_stroke_that_stops_short_misses() {
    assert!(!stroke_hits((0.0, 200.0), (200.0, 200.0), BOX, HALF));
    assert!(!stroke_hits((400.0, 200.0), (640.0, 200.0), BOX, HALF));
}

/// The point of the exact test: a long stroke cannot step over a small box the
/// way sampling it at a fixed number of places could.
#[test]
fn a_stroke_the_width_of_the_screen_cannot_step_over_the_box() {
    for along in 0..640 {
        let at = (along as f32, 200.0);
        let across = stroke_hits((0.0, 40.0), (640.0, 470.0), at, HALF);
        // The line from (0,40) to (640,470) is at y = 200 where x = 238.1.
        let near = (along as f32 - 238.1).abs() < 12.0;
        if near {
            assert!(across, "the stroke stepped over a box at {along}");
        }
    }
}

#[test]
fn a_stroke_of_no_length_is_a_point_in_or_out_of_the_box() {
    assert!(stroke_hits(BOX, BOX, BOX, HALF));
    assert!(!stroke_hits((0.0, 0.0), (0.0, 0.0), BOX, HALF));
}
