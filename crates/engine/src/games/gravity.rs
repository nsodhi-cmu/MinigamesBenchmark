//! Walk ten rooms of blocks and spikes, with nothing to steer but which way is down.

use crate::board::Board;
use crate::frame::{self, Frame};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::map::Map;
use crate::palette;
use crate::walk;

pub const SPEC: Spec = Spec {
    name: "gravity",
    title: "Go Go Gravity",
    brief: "You cannot jump. What you can do is turn gravity over, and only while you are \
            standing on something: press SPACE and you fall to the ceiling instead, or back \
            to the floor. Walk left and right and reach the green edge on the right of each \
            of ten rooms, over the ledges and past the spikes. The next room begins the way \
            the last one ended, so whichever way up you leave one is the way up you arrive \
            in the next. Touch a spike and the round is over.",
    controls: &["LEFT and RIGHT walk", "SPACE turns gravity over while you stand"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Hostile, Tag::Navigate, Tag::Timing, Tag::Platformer, Tag::Physics],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Gravity::new())
}

pub(crate) const COLUMNS: usize = 20;
pub(crate) const ROWS: usize = 14;
pub(crate) const ROOMS: usize = 10;
pub(crate) const CELL: f32 = 32.0;
const ORIGIN_X: f32 = 0.0;
const ORIGIN_Y: f32 = (frame::HEIGHT as f32 - ROWS as f32 * CELL) * 0.5;
/// Every room is the same size in the same place.
const BOARD: Board = Board::at(ORIGIN_X, ORIGIN_Y, COLUMNS, ROWS, CELL);
/// Half the side of the square you are.
pub(crate) const HALF: f32 = 10.0;
pub(crate) const PACE: f32 = 248.0;
pub(crate) const PULL: f32 = 2600.0;
pub(crate) const FASTEST: f32 = 760.0;
/// How far into a spike's square the point of it actually reaches.
const BARB: f32 = 7.0;

/// `#` a block, `^` a spike standing up, `v` one hanging down, `.` open air.
/// The first and last row of every room are solid, so there is always a floor
/// and always a ceiling. The last column is the way out.
pub(crate) const PLANS: [[&str; ROWS]; ROOMS] = [
    [
        "####################",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        ".......^^^^^^.......",
        "####################",
    ],
    [
        "####################",
        "..........vvvv......",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "...^^^^.............",
        "####################",
    ],
    [
        "####################",
        ".............#......",
        "......#......#......",
        "......#......#......",
        "......#......#......",
        "......#......#......",
        "......#......#......",
        "......#.............",
        "......#.............",
        "......#.............",
        "......#.............",
        "......#.............",
        "..^^^^..............",
        "####################",
    ],
    [
        "####################",
        ".............vvvvvv.",
        "....................",
        "....................",
        "..##########........",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "..^^^^^^^^^^........",
        "####################",
    ],
    [
        "####################",
        "..........vvvvvvvvv.",
        "....................",
        "....................",
        "....................",
        "....................",
        "......########......",
        "......vvvvvvvv......",
        "....................",
        "....................",
        "....................",
        "....................",
        "..^^^^^^...^^^^.....",
        "####################",
    ],
    [
        "####################",
        "......vvvvvvvvvvvv..",
        "....................",
        "..######............",
        "....vvvv............",
        "....................",
        "....................",
        "....................",
        "....................",
        "..........###.......",
        "....................",
        "....................",
        "..^^^^^^^^^^^.......",
        "####################",
    ],
    [
        "####################",
        "........############",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "..^^^^^^....^^^^^^..",
        "####################",
    ],
    [
        "####################",
        "..............vvvv..",
        "....................",
        "....................",
        "....................",
        ".......###..###.....",
        ".......vvv..vvv.....",
        "....................",
        "....................",
        "....................",
        "....................",
        "....................",
        "..^^^^^^^^^^........",
        "####################",
    ],
    [
        "####################",
        "..........#vvvvvvvv.",
        "..........#.........",
        "..........#.........",
        "..........#.........",
        "..........#.........",
        "....................",
        "....................",
        "......#########.....",
        "..........#.........",
        "..........#.........",
        "..........#.........",
        "..^^^^^^^^#.........",
        "####################",
    ],
    [
        "####################",
        "..vvvvvvvvvvvv......",
        "....................",
        "....................",
        "..########..........",
        "..vvvvv.............",
        "....................",
        "....................",
        "....................",
        "....................",
        "............##......",
        "....................",
        "..^^^^^^^^^^^^^^^^^.",
        "####################",
    ],
];

/// Where a body stands, and which way it falls. All the state a room hands on
/// to the next one.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Body {
    pub at: (f32, f32),
    pub drop: f32,
    /// Whether down is up: `true` means the ceiling is the floor.
    pub over: bool,
    pub standing: bool,
}

/// The room as a place, with its corner where the room is drawn.
pub(crate) fn plan(room: usize) -> Map<'static> {
    Map { rows: PLANS[room].as_slice(), board: BOARD }
}

/// Off the plan is rock: a room is closed and the way out is a mark on it.
pub(crate) fn mark(room: usize, column: i32, row: i32) -> u8 {
    plan(room).mark(column, row).unwrap_or(b'#')
}

pub(crate) fn walled(room: usize, at: (f32, f32)) -> bool {
    plan(room).touches(at, HALF, |column, row| mark(room, column, row) == b'#')
}

/// Whether a body standing at `at` is on a spike. A spike is taken to be the
/// inner part of its square, so that grazing the flat of one is not death.
pub(crate) fn spiked(room: usize, at: (f32, f32)) -> bool {
    plan(room).spanned(at, HALF).any(|(column, row)| {
        let point = mark(room, column, row);
        if point != b'^' && point != b'v' {
            return false;
        }
        let x = ORIGIN_X + column as f32 * CELL;
        let y = ORIGIN_Y + row as f32 * CELL;
        let low = if point == b'^' { y + BARB } else { y };
        at.0 + HALF > x + BARB
            && at.0 - HALF < x + CELL - BARB
            && at.1 + HALF > low
            && at.1 - HALF < low + CELL - BARB
    })
}

/// One tick of falling, walking, and turning over. Kept apart from the game so
/// that a search can play the rooms through without one.
pub(crate) fn step(room: usize, mut body: Body, way: f32, turn: bool) -> Body {
    if turn && body.standing {
        body.over = !body.over;
        body.drop = 0.0;
        body.standing = false;
    }

    let stride = way * PACE * TICK;
    let held = body.at.1;
    body.at.0 = walk::slid(body.at.0 - ORIGIN_X, stride, HALF, CELL, |x| {
        walled(room, (x + ORIGIN_X, held))
    }) + ORIGIN_X;
    body.at.0 = body.at.0.max(ORIGIN_X + HALF);

    let pull = if body.over { -PULL } else { PULL };
    body.drop = (body.drop + pull * TICK).clamp(-FASTEST, FASTEST);
    let fall = body.drop * TICK;
    let across = body.at.0;
    let above = body.at.1 - ORIGIN_Y;
    let landed = walk::slid(above, fall, HALF, CELL, |y| walled(room, (across, y + ORIGIN_Y)));
    body.standing = landed != above + fall;
    if body.standing {
        body.drop = 0.0;
    }
    body.at.1 = landed + ORIGIN_Y;
    body
}

/// Whether a body has reached the way out on the right of its room.
pub(crate) fn out(body: &Body) -> bool {
    body.at.0 >= ORIGIN_X + (COLUMNS - 1) as f32 * CELL
}

pub(crate) fn start() -> Body {
    Body {
        at: (ORIGIN_X + HALF, ORIGIN_Y + (ROWS - 1) as f32 * CELL - HALF),
        drop: 0.0,
        over: false,
        standing: true,
    }
}

struct Gravity {
    room: usize,
    body: Body,
}

impl Gravity {
    fn new() -> Self {
        Gravity { room: 0, body: start() }
    }
}

impl Game for Gravity {
    fn update(&mut self, input: &Input) -> Outcome {
        let way = input.axis(Key::Left, Key::Right);
        let turn = input.pressed(Key::Space);
        self.body = step(self.room, self.body, way, turn);

        if spiked(self.room, self.body.at) {
            return Outcome::Lost;
        }
        if out(&self.body) {
            if self.room + 1 == ROOMS {
                return Outcome::Won;
            }
            self.room += 1;
            self.body.at.0 = ORIGIN_X + HALF;
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let span = (COLUMNS as f32 * CELL, ROWS as f32 * CELL);
        frame.rect(ORIGIN_X, ORIGIN_Y, span.0, span.1, palette::BACKDROP);
        let gate = ORIGIN_X + (COLUMNS - 1) as f32 * CELL;
        frame.rect(
            gate,
            ORIGIN_Y + CELL,
            CELL,
            span.1 - CELL * 2.0,
            palette::GOOD.mix(palette::BACKDROP, 0.42),
        );

        for row in 0..ROWS as i32 {
            for column in 0..COLUMNS as i32 {
                let x = ORIGIN_X + column as f32 * CELL;
                let y = ORIGIN_Y + row as f32 * CELL;
                match mark(self.room, column, row) {
                    b'#' => {
                        frame.bordered(x, y, CELL, CELL, 2.0, palette::PANEL, palette::EDGE);
                    }
                    b'^' => frame.poly(
                        &[
                            [x + 2.0, y + CELL],
                            [x + CELL * 0.5, y + 3.0],
                            [x + CELL - 2.0, y + CELL],
                        ],
                        palette::BAD,
                    ),
                    b'v' => frame.poly(
                        &[[x + 2.0, y], [x + CELL * 0.5, y + CELL - 3.0], [x + CELL - 2.0, y]],
                        palette::BAD,
                    ),
                    _ => {}
                }
            }
        }

        let (x, y) = self.body.at;
        frame.rect(x - HALF, y - HALF, HALF * 2.0, HALF * 2.0, palette::PLAYER);
        // A bar on the side the body falls toward, so which way is down can be
        // read off a still picture.
        let foot = if self.body.over { y - HALF } else { y + HALF - 4.0 };
        frame.rect(x - HALF, foot, HALF * 2.0, 4.0, palette::MARK);
    }
}

#[cfg(test)]
#[path = "../tests/games/gravity.rs"]
mod tests;
