//! Cross a looping maze while something walks the shortest path to you.

use crate::board::Board;
use crate::frame::Frame;
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::Input;
use crate::maze::Walls as Grid;
use crate::palette;
use crate::rng::Rng;
use crate::walk;
use std::collections::VecDeque;

pub const SPEC: Spec = Spec {
    name: "maze2",
    title: "Marauder Maze",
    brief: "You are the blue dot in the top left. The red block starts on the green square \
            you are trying to reach. It slides square by square along the grid, and each \
            time it lands on one it works out the shortest way to you and takes the next \
            step of it. It is a little slower than you are, but it never stops. Touch the \
            block and the round is over; stand on the green square and you have won.",
    controls: &["arrow keys walk", "hold them to keep walking"],
    pointer: false,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Lethal, Tag::Navigate, Tag::Overhead],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Hound::new(seed))
}

const ROOMS_X: usize = 10;
const ROOMS_Y: usize = 7;
const ROOMS: usize = ROOMS_X * ROOMS_Y;
const COLUMNS: usize = 2 * ROOMS_X + 1;
const ROWS: usize = 2 * ROOMS_Y + 1;
const CELL: f32 = 28.0;
const ORIGIN: (f32, f32) = (26.0, 30.0);
const RADIUS: f32 = 8.0;
const PACE: f32 = 230.0;
const HOUND_PACE: f32 = 190.0;
const HOUND_BOX: f32 = CELL - 6.0;
const START: usize = 0;
const EXIT: usize = ROOMS - 1;
const LAIR: usize = EXIT;

pub(crate) type Walls = [[bool; COLUMNS]; ROWS];
type Maze = Grid<COLUMNS, ROWS>;

struct Hound {
    walls: Maze,
    at: (f32, f32),
    hound: (f32, f32),
    cell: usize,
    next: Option<usize>,
}

pub(crate) fn neighbors(room: usize) -> impl Iterator<Item = usize> {
    let (x, y) = (room % ROOMS_X, room / ROOMS_X);
    [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)].into_iter().filter_map(move |(dx, dy)| {
        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
        (nx >= 0 && ny >= 0 && nx < ROOMS_X as i32 && ny < ROOMS_Y as i32)
            .then(|| ny as usize * ROOMS_X + nx as usize)
    })
}

fn cell_of(room: usize) -> (usize, usize) {
    (2 * (room % ROOMS_X) + 1, 2 * (room / ROOMS_X) + 1)
}

fn between(a: usize, b: usize) -> (usize, usize) {
    let (ax, ay) = cell_of(a);
    let (bx, by) = cell_of(b);
    ((ax + bx) / 2, (ay + by) / 2)
}

fn joined(walls: &Walls, a: usize, b: usize) -> bool {
    let (x, y) = between(a, b);
    !walls[y][x]
}

pub(crate) fn links_of(walls: &Walls) -> Vec<Vec<usize>> {
    (0..ROOMS)
        .map(|room| neighbors(room).filter(|next| joined(walls, room, *next)).collect())
        .collect()
}

/// The first room that, taken out of the maze, would cut another off, by
/// Tarjan's search. A maze with none has two ways from anywhere to anywhere.
pub(crate) fn choke_point(links: &[Vec<usize>]) -> Option<usize> {
    #[allow(clippy::too_many_arguments)] // splitting the search state up buys nothing
    fn walk(
        room: usize,
        from: usize,
        links: &[Vec<usize>],
        seen: &mut [u32],
        low: &mut [u32],
        clock: &mut u32,
        choke: &mut Option<usize>,
    ) {
        *clock += 1;
        seen[room] = *clock;
        low[room] = *clock;
        let mut branches = 0;
        for &next in &links[room] {
            if next == from {
                continue;
            }
            if seen[next] != 0 {
                low[room] = low[room].min(seen[next]);
            } else {
                branches += 1;
                walk(next, room, links, seen, low, clock, choke);
                low[room] = low[room].min(low[next]);
                if from != usize::MAX && low[next] >= seen[room] {
                    *choke = choke.or(Some(room));
                }
            }
        }
        if from == usize::MAX && branches > 1 {
            *choke = choke.or(Some(room));
        }
    }

    let (mut seen, mut low, mut clock, mut choke) = ([0; ROOMS], [0; ROOMS], 0, None);
    walk(0, usize::MAX, links, &mut seen, &mut low, &mut clock, &mut choke);
    choke
}

/// Which side of `skip` each room falls on once `skip` is taken out.
fn parts_without(links: &[Vec<usize>], skip: usize) -> [usize; ROOMS] {
    let mut part = [usize::MAX; ROOMS];
    let mut next_part = 0;
    for room in 0..ROOMS {
        if room == skip || part[room] != usize::MAX {
            continue;
        }
        let mut queue = VecDeque::from([room]);
        part[room] = next_part;
        while let Some(here) = queue.pop_front() {
            for &step in &links[here] {
                if step != skip && part[step] == usize::MAX {
                    part[step] = next_part;
                    queue.push_back(step);
                }
            }
        }
        next_part += 1;
    }
    part
}

/// A maze grown as a tree, given a second way out of every dead end, and then
/// opened at exactly the walls that still leave a room with one way through.
pub(crate) fn dig(rng: &mut Rng) -> Walls {
    let mut walls = [[true; COLUMNS]; ROWS];
    let open = |walls: &mut Walls, at: (usize, usize)| walls[at.1][at.0] = false;

    let mut seen = [false; ROOMS];
    let mut stack = vec![START];
    seen[START] = true;
    open(&mut walls, cell_of(START));
    while let Some(&room) = stack.last() {
        let ways: Vec<usize> = neighbors(room).filter(|next| !seen[*next]).collect();
        let Some(&next) = ways.get(rng.below(ways.len().max(1))) else {
            stack.pop();
            continue;
        };
        open(&mut walls, between(room, next));
        open(&mut walls, cell_of(next));
        seen[next] = true;
        stack.push(next);
    }

    for room in 0..ROOMS {
        if links_of(&walls)[room].len() == 1 {
            let ways: Vec<usize> =
                neighbors(room).filter(|next| !joined(&walls, room, *next)).collect();
            if !ways.is_empty() {
                open(&mut walls, between(room, ways[rng.below(ways.len())]));
            }
        }
    }

    loop {
        let links = links_of(&walls);
        let Some(choke) = choke_point(&links) else { break };
        let parts = parts_without(&links, choke);
        let mend = (0..ROOMS).flat_map(neighbors_with).find(|(a, b)| {
            a < b && *a != choke && *b != choke && !joined(&walls, *a, *b) && parts[*a] != parts[*b]
        });
        let Some((a, b)) = mend else { break };
        open(&mut walls, between(a, b));
    }
    walls
}

fn neighbors_with(room: usize) -> impl Iterator<Item = (usize, usize)> {
    neighbors(room).map(move |next| (room.min(next), room.max(next)))
}

fn cell_neighbors(cell: usize) -> impl Iterator<Item = usize> {
    let (x, y) = (cell % COLUMNS, cell / COLUMNS);
    [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)].into_iter().filter_map(move |(dx, dy)| {
        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
        (nx >= 0 && ny >= 0 && nx < COLUMNS as i32 && ny < ROWS as i32)
            .then(|| ny as usize * COLUMNS + nx as usize)
    })
}

impl Hound {
    fn new(seed: u64) -> Self {
        let board = Board::at(ORIGIN.0, ORIGIN.1, COLUMNS, ROWS, CELL);
        let walls = Grid { solid: dig(&mut Rng::new(seed)), board };
        let (lx, ly) = cell_of(LAIR);
        let (sx, sy) = cell_of(START);
        let (at, hound) = (walls.board.middle(sx, sy), walls.board.middle(lx, ly));
        Hound { walls, at, hound, cell: ly * COLUMNS + lx, next: None }
    }

    fn cell_middle(&self, cell: usize) -> (f32, f32) {
        self.walls.board.middle(cell % COLUMNS, cell / COLUMNS)
    }

    fn room_middle(&self, room: usize) -> (f32, f32) {
        let (column, row) = cell_of(room);
        self.walls.board.middle(column, row)
    }
}

impl Game for Hound {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some(step) = input.heading() {
            self.at =
                walk::walked(self.at, step, PACE * TICK, |x, y| self.walls.clear((x, y), RADIUS));
        }

        if self.next.is_none() {
            let steps = self.walls.steps(self.walls.board.nearest(self.at));
            self.next = cell_neighbors(self.cell)
                .filter(|cell| !self.walls.solid[cell / COLUMNS][cell % COLUMNS])
                .min_by_key(|cell| steps[*cell])
                .filter(|cell| steps[*cell] < steps[self.cell]);
        }
        if let Some(next) = self.next {
            let goal = self.cell_middle(next);
            self.hound = walk::toward(self.hound, goal, HOUND_PACE * TICK);
            if self.hound == goal {
                self.cell = next;
                self.next = None;
            }
        }

        let half = HOUND_BOX * 0.5;
        let near = (
            self.at.0.clamp(self.hound.0 - half, self.hound.0 + half),
            self.at.1.clamp(self.hound.1 - half, self.hound.1 + half),
        );
        let (gx, gy) = (self.at.0 - near.0, self.at.1 - near.1);
        if gx * gx + gy * gy < RADIUS * RADIUS {
            return Outcome::Lost;
        }

        let exit = self.room_middle(EXIT);
        let (ex, ey) = (self.at.0 - exit.0, self.at.1 - exit.1);
        if ex * ex + ey * ey < CELL * CELL * 0.16 { Outcome::Won } else { Outcome::Playing }
    }

    fn draw(&self, frame: &mut Frame) {
        self.walls.draw(frame);

        let (ex, ey) = self.room_middle(EXIT);
        frame.rect(ex - CELL * 0.35, ey - CELL * 0.35, CELL * 0.7, CELL * 0.7, palette::GOOD);

        let (hx, hy) = self.hound;
        let half = HOUND_BOX * 0.5;
        frame.rect(hx - half, hy - half, HOUND_BOX, HOUND_BOX, palette::BAD);
        frame.cross(hx, hy, 5.0, 2.0, palette::BACKDROP);
        frame.disc(self.at.0, self.at.1, RADIUS, palette::PLAYER);
    }
}

#[cfg(test)]
#[path = "../tests/games/maze2.rs"]
mod tests;
