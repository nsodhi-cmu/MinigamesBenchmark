use std::collections::{HashMap, VecDeque};

use super::{BALL, Ball, FALL, MAZE, Maze, RINGS, Rings, SECTORS, SPIN, WHOLE, lane};
use crate::game::{Game, LIMIT, Outcome, TICK_HZ};
use crate::input::{Action, Input, Key};
use crate::playing::LOOK;
use crate::rng::Rng;

/// A turn one way or the other, or none, held for a whole look.
const WAYS: [i32; 3] = [0, 1, -1];

fn held(way: i32) -> Input {
    let mut input = Input::new();
    match way {
        1 => input.apply(Action::Down(Key::Right)),
        -1 => input.apply(Action::Down(Key::Left)),
        _ => {}
    }
    input
}

/// Everything a maze is, worked out from the whole graph of states a player
/// can reach in it, holding one way or neither for `grain` ticks at a time.
struct Report {
    /// The fewest looks out, if there is a way out at all.
    fastest: Option<Vec<i32>>,
    /// How many of the states a player can reach have no way out from them.
    dead_ends: usize,
    reached: usize,
    /// Whether holding one way from the start ever gets out.
    one_way_wins: bool,
}

type State = (Ball, i32);

fn step(maze: &Maze, state: State, way: i32, grain: u32) -> State {
    let (mut ball, mut turn) = state;
    for _ in 0..grain {
        turn = (turn + way * SPIN).rem_euclid(WHOLE);
        ball = maze.tick(ball, turn);
    }
    (ball, turn)
}

fn analyze(maze: &Maze, grain: u32) -> Report {
    let start = (Ball::At { ring: 0, angle: WHOLE / 4 }, 0);
    let mut index: HashMap<State, u32> = HashMap::from([(start, 0)]);
    let mut states = vec![start];
    let mut next: Vec<[u32; 3]> = Vec::new();
    let mut came: Vec<(u32, i32)> = vec![(0, 0)];
    let mut queue = VecDeque::from([0u32]);
    while let Some(at) = queue.pop_front() {
        let state = states[at as usize];
        let mut row = [0u32; 3];
        for (which, way) in WAYS.iter().enumerate() {
            let to = if state.0 == Ball::Out { state } else { step(maze, state, *way, grain) };
            let id = *index.entry(to).or_insert_with(|| {
                states.push(to);
                came.push((at, *way));
                queue.push_back(states.len() as u32 - 1);
                states.len() as u32 - 1
            });
            row[which] = id;
        }
        next.push(row);
    }
    let out: Vec<u32> =
        (0..states.len() as u32).filter(|i| states[*i as usize].0 == Ball::Out).collect();
    let fastest = out.iter().min().map(|&goal| {
        let mut ways = Vec::new();
        let mut at = goal;
        while at != 0 {
            let (before, way) = came[at as usize];
            ways.push(way);
            at = before;
        }
        ways.reverse();
        ways
    });
    let mut back: Vec<Vec<u32>> = vec![Vec::new(); states.len()];
    for (from, row) in next.iter().enumerate() {
        for to in row {
            back[*to as usize].push(from as u32);
        }
    }
    let mut wins = vec![false; states.len()];
    let mut queue: VecDeque<u32> = out.iter().copied().collect();
    for o in &out {
        wins[*o as usize] = true;
    }
    while let Some(at) = queue.pop_front() {
        for from in &back[at as usize] {
            if !wins[*from as usize] {
                wins[*from as usize] = true;
                queue.push_back(*from);
            }
        }
    }
    let one_way_wins = [1, -1].iter().any(|way| {
        let mut state = start;
        (0..(LIMIT * TICK_HZ as f32) as u32 / grain).any(|_| {
            state = step(maze, state, *way, grain);
            state.0 == Ball::Out
        })
    });
    Report {
        fastest,
        dead_ends: wins.iter().filter(|w| !**w).count(),
        reached: states.len(),
        one_way_wins,
    }
}

/// How many times the ball falls back toward the middle along a way out.
fn climbs(ways: &[i32], grain: u32) -> usize {
    let mut game = Rings::new();
    let mut climbs = 0;
    for way in ways {
        for _ in 0..grain {
            game.update(&held(*way));
            if let Ball::Falling { from, to, left, .. } = game.ball
                && left == FALL
                && to < from
            {
                climbs += 1;
            }
        }
    }
    climbs
}

/// Holding one way never gets the ball out. Wherever a player can get the ball
/// there is still a way out from there, and even played a tick at a time the
/// fastest way out has to drop the ball back toward the middle first.
#[test]
fn the_maze_has_no_dead_end_and_no_way_out_without_turning_back_and_falling_inward() {
    let report = analyze(&MAZE, 1);
    assert!(!report.one_way_wins, "holding one way gets the ball out");
    assert_eq!(
        report.dead_ends, 0,
        "{} of {} reachable states are dead ends",
        report.dead_ends, report.reached
    );
    let ways = report.fastest.expect("the maze has a way out");
    assert!(climbs(&ways, 1) >= 1, "the fastest way out never falls inward");
}

/// A player who changes what they hold only once a tenth of a second can get the
/// ball out, and doing what they do through the game wins it.
#[test]
fn a_player_who_looks_ten_times_a_second_can_get_the_ball_out() {
    let ways = analyze(&MAZE, LOOK).fastest.expect("the maze has a way out at a tenth of a second");
    let mut game = Rings::new();
    let mut outcome = Outcome::Playing;
    'look: for way in &ways {
        for _ in 0..LOOK {
            outcome = game.update(&held(*way));
            if outcome == Outcome::Won {
                break 'look;
            }
        }
    }
    assert_eq!(outcome, Outcome::Won);
}

/// However the maze is turned, the ball is never drawn over a spoke: it stops a
/// ball's width short of every one, and lands clear of them.
#[test]
fn the_ball_never_overlaps_a_spoke() {
    for seed in 0..8u64 {
        let mut rng = Rng::new(seed);
        let mut game = Rings::new();
        let mut way = 0;
        for tick in 0..(LIMIT * TICK_HZ as f32) as u32 {
            if tick % 20 == 0 {
                way = WAYS[rng.below(WAYS.len())];
            }
            if game.update(&held(way)) == Outcome::Won {
                break;
            }
            let Ball::At { ring, angle } = game.ball else { continue };
            if ring == 0 || ring > RINGS {
                continue;
            }
            for sector in (0..SECTORS).filter(|s| MAZE.spokes[ring - 1] >> s & 1 == 1) {
                let spoke = (sector as i32 + 1) * (WHOLE / SECTORS as i32);
                let off = ((angle - spoke + WHOLE / 2).rem_euclid(WHOLE) - WHOLE / 2).abs();
                let pixels = 2.0 * lane(ring) * libm::sinf((off as f32 / 20.0).to_radians());
                assert!(
                    pixels >= BALL,
                    "seed {seed} tick {tick}: the ball sits {pixels:.1} from a spoke on ring {ring}"
                );
            }
        }
    }
}
