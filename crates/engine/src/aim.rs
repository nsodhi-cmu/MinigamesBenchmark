//! Drawing a throw back from an anchor and letting it go.

use crate::input::{Button, Input};

/// The speed a throw let go of at `at` would leave `anchor` with: the way back
/// to the anchor, `pull` times as fast, and never faster than `fastest`.
pub fn cast(anchor: (f32, f32), at: (f32, f32), pull: f32, fastest: f32) -> (f32, f32) {
    let back = ((anchor.0 - at.0) * pull, (anchor.1 - at.1) * pull);
    let fast = libm::hypotf(back.0, back.1);
    if fast > fastest { (back.0 * fastest / fast, back.1 * fastest / fast) } else { back }
}

/// Follows a drag back from `anchor`, which is begun by pressing within `grab`
/// of it. `drawing` is where the pointer has gotten to while one is on, for the
/// game to draw. The answer is where the drag was let go of, on the one tick
/// it ends.
pub fn drawn(
    input: &Input,
    anchor: (f32, f32),
    grab: f32,
    drawing: &mut Option<(f32, f32)>,
) -> Option<(f32, f32)> {
    let at = input.cursor();
    if input.button_pressed(Button::Left) && libm::hypotf(at.0 - anchor.0, at.1 - anchor.1) < grab {
        *drawing = Some(at);
    }
    if drawing.is_none() {
        return None;
    }
    *drawing = Some(at);
    if input.button_released(Button::Left) {
        *drawing = None;
        return Some(at);
    }
    None
}
