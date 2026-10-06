//! Whether things touch, for games that have to decide it exactly.

/// Whether the stroke from `a` to `b` touches the box `half` across and `half`
/// down about `at`.
///
/// Exact, by clipping the stroke against each pair of the box's sides in turn
/// and keeping the stretch of it that is inside both. Sampling a stroke at so
/// many points instead would let a long one step over a small box, and a
/// pointer that is put somewhere rather than dragged there makes every stroke
/// long.
pub fn stroke_hits(a: (f32, f32), b: (f32, f32), at: (f32, f32), half: (f32, f32)) -> bool {
    let (mut inside, mut until) = (0.0f32, 1.0f32);
    for (from, step, middle, half) in
        [(a.0, b.0 - a.0, at.0, half.0), (a.1, b.1 - a.1, at.1, half.1)]
    {
        let (low, high) = (middle - half, middle + half);
        if step == 0.0 {
            if from < low || from > high {
                return false;
            }
            continue;
        }
        let (near, far) = ((low - from) / step, (high - from) / step);
        inside = inside.max(near.min(far));
        until = until.min(near.max(far));
        if inside > until {
            return false;
        }
    }
    true
}

#[cfg(test)]
#[path = "tests/shape.rs"]
mod tests;
