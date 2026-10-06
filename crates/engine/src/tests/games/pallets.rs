use std::collections::{HashMap, VecDeque};

use super::{Layout, Pallets, ROOMS, SIDE, beside};
use crate::game::{Game, Outcome};
use crate::input::{ARROWS, Action, Input, Key};

fn open(layout: &Layout, cell: usize) -> bool {
    (layout.walls | layout.crates) >> cell & 1 == 0
}

/// The way to walk from where the player stands to `to` without pushing
/// anything, as arrows, if there is one.
fn route(layout: &Layout, to: usize) -> Option<Vec<Key>> {
    let mut came: HashMap<usize, (usize, Key)> = HashMap::new();
    let mut queue = VecDeque::from([layout.player]);
    while let Some(cell) = queue.pop_front() {
        if cell == to {
            let mut arrows = Vec::new();
            let mut at = to;
            while at != layout.player {
                let (from, arrow) = came[&at];
                arrows.push(arrow);
                at = from;
            }
            arrows.reverse();
            return Some(arrows);
        }
        for (arrow, _) in ARROWS {
            let Some(next) = beside(cell, arrow) else { continue };
            if open(layout, next) && next != layout.player && !came.contains_key(&next) {
                came.insert(next, (cell, arrow));
                queue.push_back(next);
            }
        }
    }
    None
}

/// A room as far as pushing is concerned: the crates, and the first square of
/// the part of the floor the player can walk about in.
fn arrangement(layout: &Layout) -> (u64, usize) {
    let walkable = (0..SIDE * SIDE).find(|cell| route(layout, *cell).is_some()).unwrap();
    (layout.crates, walkable)
}

/// Every layout one push away, each with the square it was pushed from and the way.
fn pushes(layout: &Layout) -> Vec<(usize, Key, Layout)> {
    let mut out = Vec::new();
    for from in (0..SIDE * SIDE).filter(|cell| route(layout, *cell).is_some()) {
        for (arrow, _) in ARROWS {
            if beside(from, arrow).is_some_and(|crate_at| layout.crates >> crate_at & 1 == 1) {
                let pushed = Layout { player: from, ..*layout }.walked(arrow);
                if pushed.crates != layout.crates {
                    out.push((from, arrow, pushed));
                }
            }
        }
    }
    out
}

/// Every arrangement a room can reach, and how many pushes it is from
/// finishing, where it can be finished at all.
struct Reach {
    layouts: Vec<Layout>,
    to_finish: Vec<Option<u32>>,
}

fn reach(room: usize) -> Reach {
    let start = Layout::of(room);
    let mut index = HashMap::from([(arrangement(&start), 0)]);
    let mut layouts = vec![start];
    let mut back: Vec<Vec<usize>> = vec![Vec::new()];
    let mut queue = VecDeque::from([0]);
    while let Some(at) = queue.pop_front() {
        for (_, _, pushed) in pushes(&layouts[at]) {
            let id = *index.entry(arrangement(&pushed)).or_insert_with(|| {
                layouts.push(pushed);
                back.push(Vec::new());
                queue.push_back(layouts.len() - 1);
                layouts.len() - 1
            });
            back[id].push(at);
        }
    }
    let mut to_finish = vec![None; layouts.len()];
    let mut queue: VecDeque<usize> =
        (0..layouts.len()).filter(|id| layouts[*id].crates == layouts[*id].marks).collect();
    for id in &queue {
        to_finish[*id] = Some(0);
    }
    while let Some(at) = queue.pop_front() {
        for before in back[at].clone() {
            if to_finish[before].is_none() {
                to_finish[before] = Some(to_finish[at].unwrap() + 1);
                queue.push_back(before);
            }
        }
    }
    Reach { layouts, to_finish }
}

/// Walking to each push of a shortest answer and pressing into it clears all
/// three rooms.
#[test]
fn walking_and_pushing_each_shortest_answer_wins() {
    let mut game = Pallets::new();
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    for room in 0..ROOMS.len() {
        assert_eq!(game.room, room);
        let reach = reach(room);
        let mut left = reach.to_finish[0].unwrap();
        while left > 0 {
            let (from, arrow, _) = pushes(&game.layout)
                .into_iter()
                .find(|(_, _, pushed)| {
                    let id =
                        reach.layouts.iter().position(|l| arrangement(l) == arrangement(pushed));
                    id.is_some_and(|id| reach.to_finish[id] == Some(left - 1))
                })
                .expect("a shortest answer carries on");
            for key in route(&game.layout, from).unwrap().into_iter().chain([arrow]) {
                assert_eq!(outcome, Outcome::Playing);
                input.apply(Action::Tap(key));
                outcome = game.update(&input);
                input.settle();
            }
            left -= 1;
        }
    }
    assert_eq!(outcome, Outcome::Won);
}
