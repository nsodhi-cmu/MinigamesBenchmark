//! Slide the cars and trucks aside until the red car can leave by the gap.

use crate::board::Board;
use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;

pub const SPEC: Spec = Spec {
    name: "jam",
    title: "Jam Jostle",
    brief: "A six by six lot is packed with cars two squares long and trucks three squares \
            long. Each one slides only the way it points, forward or back, and none can pass \
            through another. Drag them out of the way until the red car can drive out of the \
            gap in the right side of the third row, then drag it out through the gap. There \
            are three lots.",
    controls: &["press on a car or truck and drag it along the way it points"],
    pointer: true,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Unbounded, Tag::Friendly, Tag::Puzzle, Tag::Grid, Tag::Drag],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Jam::new())
}

pub(crate) const SIDE: usize = 6;
pub(crate) const EXIT_ROW: usize = 2;
pub(crate) const CELL: f32 = 64.0;
pub(crate) const BOARD: Board = Board::centered(SIDE, SIDE, CELL);
const INSET: f32 = 5.0;

/// Each letter is one car or truck, `R` the red car.
pub(crate) const BOARDS: [[&str; SIDE]; 3] = [
    ["DD.FFF", ".HH.C.", "..RRCJ", "GBBB.J", "G.EAA.", "..E.II"],
    ["III.D.", "..BBD.", "RR.EDA", "KJJE.A", "KHFFGG", ".H.CC."],
    ["..BBBJ", ".DDD.J", "RREG.F", "IIEG.F", "A..GCC", "AHH..."],
];

const RED: Rgb = Rgb(196, 30, 36);
const GRAYS: [Rgb; 12] = [
    Rgb(139, 156, 144),
    Rgb(129, 117, 138),
    Rgb(116, 114, 99),
    Rgb(139, 154, 156),
    Rgb(138, 117, 128),
    Rgb(103, 116, 99),
    Rgb(140, 139, 156),
    Rgb(138, 124, 117),
    Rgb(99, 116, 109),
    Rgb(155, 139, 156),
    Rgb(133, 138, 117),
    Rgb(99, 107, 116),
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Vehicle {
    pub(crate) across: bool,
    /// The row it runs along if it lies across, or the column if it stands up.
    pub(crate) lane: usize,
    /// The first square it covers along its lane.
    pub(crate) at: usize,
    pub(crate) length: usize,
}

impl Vehicle {
    pub(crate) fn covers(&self, cell: usize) -> bool {
        let (row, column) = (cell / SIDE, cell % SIDE);
        let (lane, along) = if self.across { (row, column) } else { (column, row) };
        lane == self.lane && (self.at..self.at + self.length).contains(&along)
    }
}

/// The vehicles of a board, the red car first.
pub(crate) fn parked(board: usize) -> Vec<Vehicle> {
    let letters: String = BOARDS[board].concat();
    let mut names: Vec<char> = letters.chars().filter(|c| *c != '.').collect();
    names.sort_by_key(|c| (*c != 'R', *c));
    names.dedup();
    names
        .into_iter()
        .map(|name| {
            let cells: Vec<usize> = letters.match_indices(name).map(|(at, _)| at).collect();
            let across = cells[0] / SIDE == cells[1] / SIDE;
            let (lane, at) = if across {
                (cells[0] / SIDE, cells[0] % SIDE)
            } else {
                (cells[0] % SIDE, cells[0] / SIDE)
            };
            Vehicle { across, lane, at, length: cells.len() }
        })
        .collect()
}

/// How far a vehicle could stand along its lane, least and most, without
/// passing through another. The red car's most runs on out through the gap.
pub(crate) fn reach(vehicles: &[Vehicle], which: usize) -> (usize, usize) {
    let me = vehicles[which];
    let free = |along: usize| {
        let cell = if me.across { me.lane * SIDE + along } else { along * SIDE + me.lane };
        vehicles.iter().enumerate().all(|(other, v)| other == which || !v.covers(cell))
    };
    let mut low = me.at;
    while low > 0 && free(low - 1) {
        low -= 1;
    }
    let mut high = me.at;
    while high + me.length < SIDE && free(high + me.length) {
        high += 1;
    }
    if which == 0 && high + me.length == SIDE {
        high = SIDE;
    }
    (low, high)
}

struct Grab {
    vehicle: usize,
    /// Where along the lane the pointer was, in squares, less where the vehicle was.
    hold: f32,
    low: usize,
    high: usize,
    along: f32,
}

pub(crate) struct Jam {
    pub(crate) board: usize,
    pub(crate) vehicles: Vec<Vehicle>,
    grab: Option<Grab>,
}

impl Jam {
    pub(crate) fn new() -> Self {
        Jam { board: 0, vehicles: parked(0), grab: None }
    }
}

/// The pointer's place in squares from the lot's corner, across then down.
fn squares(at: (f32, f32)) -> (f32, f32) {
    ((at.0 - BOARD.left) / CELL, (at.1 - BOARD.top) / CELL)
}

impl Game for Jam {
    fn update(&mut self, input: &Input) -> Outcome {
        let (x, y) = squares(input.cursor());
        if input.button_pressed(Button::Left)
            && let Some((column, row)) = BOARD.under(input.cursor())
        {
            let cell = row * SIDE + column;
            self.grab = self.vehicles.iter().position(|v| v.covers(cell)).map(|vehicle| {
                let v = self.vehicles[vehicle];
                let (low, high) = reach(&self.vehicles, vehicle);
                let pointer = if v.across { x } else { y };
                Grab { vehicle, hold: pointer - v.at as f32, low, high, along: v.at as f32 }
            });
        }
        let Some(grab) = &mut self.grab else { return Outcome::Playing };
        let v = &mut self.vehicles[grab.vehicle];
        let pointer = if v.across { x } else { y };
        grab.along = (pointer - grab.hold).clamp(grab.low as f32, grab.high as f32);
        if input.button_held(Button::Left) {
            return Outcome::Playing;
        }

        v.at = grab.along.round() as usize;
        let out = grab.vehicle == 0 && v.at + 1 >= SIDE;
        self.grab = None;
        if !out {
            return Outcome::Playing;
        }
        if self.board + 1 == BOARDS.len() {
            return Outcome::Won;
        }
        self.board += 1;
        self.vehicles = parked(self.board);
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let (across, down) = BOARD.span();
        frame.rect(BOARD.left - 6.0, BOARD.top - 6.0, across + 12.0, down + 12.0, palette::EDGE);
        frame.rect(BOARD.left, BOARD.top, across, down, palette::PANEL);
        frame.rect(
            BOARD.left + across,
            BOARD.top + EXIT_ROW as f32 * CELL,
            6.0,
            CELL,
            palette::BACKDROP,
        );
        for step in 1..SIDE {
            let along = step as f32 * CELL;
            frame.rect(BOARD.left + along - 1.0, BOARD.top, 2.0, down, palette::EDGE);
            frame.rect(BOARD.left, BOARD.top + along - 1.0, across, 2.0, palette::EDGE);
        }
        let arrow = (BOARD.left + across + 26.0, BOARD.top + (EXIT_ROW as f32 + 0.5) * CELL);
        frame.poly(
            &[[arrow.0, arrow.1 - 12.0], [arrow.0 + 14.0, arrow.1], [arrow.0, arrow.1 + 12.0]],
            palette::MUTED,
        );

        for (which, v) in self.vehicles.iter().enumerate() {
            let along = match &self.grab {
                Some(grab) if grab.vehicle == which => grab.along,
                _ => v.at as f32,
            };
            let (column, row) =
                if v.across { (along, v.lane as f32) } else { (v.lane as f32, along) };
            let (wide, tall) =
                if v.across { (v.length as f32, 1.0) } else { (1.0, v.length as f32) };
            let tint = if which == 0 { RED } else { GRAYS[(which - 1) % GRAYS.len()] };
            frame.rect(
                BOARD.left + column * CELL + INSET,
                BOARD.top + row * CELL + INSET,
                wide * CELL - INSET * 2.0,
                tall * CELL - INSET * 2.0,
                tint,
            );
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/jam.rs"]
mod tests;
