use super::{FALL, LASTS, Rocks, SIDE, Square, square_of};
use crate::game::{Game, Outcome};
use crate::input::{Input, Key};
use crate::playing::{LOOK, holding};

const WAYS: [&[Key]; 9] = [
    &[],
    &[Key::Left],
    &[Key::Right],
    &[Key::Up],
    &[Key::Down],
    &[Key::Left, Key::Up],
    &[Key::Right, Key::Up],
    &[Key::Left, Key::Down],
    &[Key::Right, Key::Down],
];

/// Play one way for some looks and then stand still, until every rock already
/// on its way has landed. Nothing that starts falling in that time lands
/// inside it, so this sees only what is on the screen.
fn survives(game: &Rocks, way: &Input, looks: u32) -> Option<Rocks> {
    let mut game = game.clone();
    let still = Input::new();
    for tick in 0..FALL {
        let input = if tick < looks * LOOK { way } else { &still };
        if game.update(input) == Outcome::Lost {
            return None;
        }
    }
    Some(game)
}

/// A player who looks ten times a second and knows nothing the screen does not
/// show them lasts the half minute on every seed.
#[test]
fn a_player_who_reads_the_falling_squares_lasts_the_half_minute() {
    for seed in 0..30u64 {
        let mut game = Rocks::new(seed);
        let mut outcome = Outcome::Playing;
        while outcome == Outcome::Playing {
            let mut best: Option<(f32, usize)> = None;
            for (which, keys) in WAYS.iter().enumerate() {
                let way = holding(keys);
                for looks in 1..=10 {
                    let Some(after) = survives(&game, &way, looks) else { continue };
                    let square = square_of(after.at);
                    let middle = super::BOARD.middle(square % SIDE, square / SIDE);
                    let exposed = if matches!(after.squares[square], Square::Falling(_)) {
                        1000.0
                    } else {
                        0.0
                    };
                    let score =
                        exposed + libm::hypotf(after.at.0 - middle.0, after.at.1 - middle.1);
                    if best.is_none_or(|(least, _)| score < least) {
                        best = Some((score, which));
                    }
                }
            }
            let Some((_, which)) = best else {
                panic!("seed {seed} left no way out at tick {}", game.tick);
            };
            let way = holding(WAYS[which]);
            for _ in 0..LOOK {
                outcome = game.update(&way);
                if outcome != Outcome::Playing {
                    break;
                }
            }
        }
        assert_eq!(outcome, Outcome::Won, "seed {seed} crushed a careful player");
        assert!(game.tick >= LASTS);
    }
}
