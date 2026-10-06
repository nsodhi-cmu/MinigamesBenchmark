use super::{ROAD, Race, TRACK, VERGE, mark_at, off_line};
use crate::game::{Game, LIMIT, Outcome, TICK_HZ};
use crate::input::{Action, Input, Key};

/// How far apart two points of the track are along it, the shorter way
/// around, which is what tells a tight corner from a place where the course
/// has doubled back on itself.
fn along(one: usize, two: usize) -> f32 {
    let leg = |i: usize| {
        let (a, b) = (TRACK[i], TRACK[(i + 1) % TRACK.len()]);
        libm::hypotf(a[0] - b[0], a[1] - b[1])
    };
    let whole: f32 = (0..TRACK.len()).map(leg).sum();
    let forward: f32 =
        (0..(two + TRACK.len() - one) % TRACK.len()).map(|k| leg((one + k) % TRACK.len())).sum();
    forward.min(whole - forward)
}

/// Where the course comes back near itself it is a corner and not a doubling
/// back, so the walls never meet and there is no way across the middle.
#[test]
fn the_track_never_doubles_back_within_reach_of_itself() {
    for (i, a) in TRACK.iter().enumerate() {
        for (j, b) in TRACK.iter().enumerate() {
            if i == j || along(i, j) <= 200.0 {
                continue;
            }
            let gap = libm::hypotf(a[0] - b[0], a[1] - b[1]);
            assert!(gap > VERGE, "the track comes back within {gap:.0} of itself at {i} and {j}");
        }
    }
}

/// The whole course fits on the screen with its walls inside it.
#[test]
fn the_course_and_its_walls_sit_inside_the_picture() {
    use crate::frame;
    for point in TRACK {
        assert!(point[0] - VERGE * 0.5 > 0.0, "the track runs off the left");
        assert!(point[0] + VERGE * 0.5 < frame::WIDTH as f32, "the track runs off the right");
        assert!(point[1] - VERGE * 0.5 > 0.0, "the track runs off the top of the screen");
        assert!(point[1] + VERGE * 0.5 < frame::HEIGHT as f32, "the track runs off the bottom");
    }
}

/// Every mark sits on the road rather than on the grass beside it.
#[test]
fn every_mark_sits_on_the_road() {
    for number in 0..4 {
        assert!(off_line(mark_at(number)) < ROAD * 0.5, "mark {number} is off the road");
    }
}

/// Steering around the middle of the road finishes a lap.
#[test]
fn following_the_middle_of_the_road_finishes_a_lap() {
    let mut game = Race::new();
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    for _ in 0..LIMIT as u32 * TICK_HZ {
        // Follow the middle of the road: when off it, head straight back to
        // the nearest point of it; when on it, aim a way farther around.
        let (leg, spot) = (0..TRACK.len())
            .map(|i| {
                let (a, b) = (TRACK[i], TRACK[(i + 1) % TRACK.len()]);
                let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                let along = dx * dx + dy * dy;
                let part =
                    (((game.at.0 - a[0]) * dx + (game.at.1 - a[1]) * dy) / along).clamp(0.0, 1.0);
                (i, [a[0] + dx * part, a[1] + dy * part])
            })
            .min_by(|one, two| {
                let off = |p: &[f32; 2]| libm::hypotf(p[0] - game.at.0, p[1] - game.at.1);
                off(&one.1).total_cmp(&off(&two.1))
            })
            .expect("the track has legs");
        let ahead =
            if off_line(game.at) > ROAD * 0.4 { spot } else { TRACK[(leg + 2) % TRACK.len()] };
        let want = libm::atan2f(ahead[1] - game.at.1, ahead[0] - game.at.0);
        let mut off = want - game.facing;
        while off > std::f32::consts::PI {
            off -= std::f32::consts::TAU;
        }
        while off < -std::f32::consts::PI {
            off += std::f32::consts::TAU;
        }
        input.clear();
        input.apply(Action::Down(if off > 0.0 { Key::Right } else { Key::Left }));
        if off.abs() < 0.4 {
            input.apply(Action::Down(Key::Space));
        }
        outcome = game.update(&input);
        input.settle();
        if outcome == Outcome::Won {
            return;
        }
        assert!(off_line(game.at) < VERGE * 0.5, "the driver left the course");
    }
    assert_eq!(outcome, Outcome::Won, "a lap was never finished");
}
