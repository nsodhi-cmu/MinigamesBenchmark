//! What a test needs to play a game: the pace it looks at the screen, the keys
//! it holds, and a search for the fewest looks through a place.

use std::collections::HashSet;
use std::hash::Hash;

use crate::game::TICK_HZ;
use crate::input::{Action, Input, Key};

/// A tenth of a second of game time, one look for a player that looks ten
/// times a second.
pub const LOOK: u32 = TICK_HZ / 10;

/// An input with these keys held down and nothing else.
pub fn holding(keys: &[Key]) -> Input {
    let mut input = Input::new();
    for key in keys {
        input.apply(Action::Down(*key));
    }
    input
}

/// What one tick of one move came to.
pub enum Went<S> {
    On(S),
    /// Out of the place, which is what the search is looking for.
    Out(S),
    /// Dead, so this move is no move.
    Dead,
}

/// The fewest looks out of a place, for a player who holds the same keys for a
/// whole look at a time: which move each look was, and where it left them.
///
/// `step` is one tick of one move, told how many ticks have passed since the
/// start and which of the `moves` is being held. `key` says when two states
/// are near enough to count as one, which is what keeps the search finite.
pub fn fewest<S: Copy, K: Eq + Hash>(
    start: S,
    moves: usize,
    key: impl Fn(S, u32) -> K,
    step: impl Fn(S, u32, usize) -> Went<S>,
) -> Option<Vec<(usize, S)>> {
    let mut nodes = vec![(start, 0u32, usize::MAX, 0usize)];
    let mut seen = HashSet::from([key(start, 0)]);
    let mut next = 0;
    while next < nodes.len() {
        let (from, tick, _, _) = nodes[next];
        'moves: for which in 0..moves {
            let (mut at, mut now) = (from, tick);
            for _ in 0..LOOK {
                match step(at, now, which) {
                    Went::On(gone) => at = gone,
                    Went::Out(gone) => {
                        let mut route = vec![(which, gone)];
                        let mut back = next;
                        while nodes[back].2 != usize::MAX {
                            route.push((nodes[back].3, nodes[back].0));
                            back = nodes[back].2;
                        }
                        route.reverse();
                        return Some(route);
                    }
                    Went::Dead => continue 'moves,
                }
                now += 1;
            }
            if seen.insert(key(at, now)) {
                nodes.push((at, now, next, which));
            }
        }
        next += 1;
    }
    None
}
