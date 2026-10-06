use super::{KEYS, LASTS, Tiles};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input};

/// Striking every tile on the tick it asks for, and holding the long ones to
/// their far end, must last the thirty seconds. If the chart ever laid two
/// tiles in one lane close enough to be confused, this would not.
#[test]
fn playing_a_chart_exactly_lasts_the_whole_thirty_seconds() {
    for seed in 0..12u64 {
        let mut game = Tiles::new(seed);
        let mut input = Input::new();

        let mut plan: Vec<(u32, usize, bool)> = Vec::new();
        for tile in &game.tiles {
            plan.push((tile.at, tile.lane, true));
            plan.push((
                if tile.until > tile.at { tile.until } else { tile.at + 1 },
                tile.lane,
                false,
            ));
        }

        let mut outcome = Outcome::Playing;
        for tick in 1..=LASTS {
            for (at, lane, down) in &plan {
                if *at == tick {
                    input.apply(if *down {
                        Action::Down(KEYS[*lane])
                    } else {
                        Action::Up(KEYS[*lane])
                    });
                }
            }
            outcome = game.update(&input);
            input.settle();
            if outcome != Outcome::Playing {
                break;
            }
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} could not be played through");
    }
}

/// Two tiles in one lane must sit far enough apart that a strike cannot be
/// taken for the wrong one.
#[test]
fn no_two_tiles_in_a_lane_sit_inside_each_others_leeway() {
    for seed in 0..40u64 {
        let game = Tiles::new(seed);
        for lane in 0..super::LANES {
            let mut times: Vec<(u32, u32)> =
                game.tiles.iter().filter(|t| t.lane == lane).map(|t| (t.at, t.until)).collect();
            times.sort_unstable();
            for pair in times.windows(2) {
                let gap = pair[1].0 - pair[0].1;
                assert!(gap > super::LEEWAY * 2, "seed {seed} lane {lane} left a gap of {gap}");
            }
        }
    }
}
