//! A four by four floor that rocks keep falling onto, and half a minute to stay out from under them.

use crate::board::Board;
use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, TICK_HZ, Tag};
use crate::input::Input;
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "rocks",
    title: "Falling Rocks",
    brief: "You are the dot on a floor of sixteen squares. About every half a second one \
            square is boxed in yellow and a rock starts falling onto it: its shadow darkens \
            and the rock closes in on the square, landing one second after the box appears. \
            Be anywhere else when it lands. If the middle of the dot is inside the square \
            when the rock lands, the round is over. A rock that has landed is harmless and \
            crumbles away. Last thirty seconds to finish.",
    controls: &["arrow keys move, diagonals too"],
    pointer: false,
    scoring: Scoring::Boolean,
    varies: true,
    tags: &[Tag::Fixed, Tag::Lethal, Tag::Timing, Tag::Overhead, Tag::Grid],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Rocks::new(seed))
}

pub(crate) const SIDE: usize = 4;
pub(crate) const REGION: f32 = 96.0;
pub(crate) const BOARD: Board = Board::at(128.0, 64.0, SIDE, SIDE, REGION);
const DOT: f32 = 7.0;
/// Pixels a second.
const PACE: f32 = 180.0;
pub(crate) const FALL: u32 = TICK_HZ;
const CRUMBLE: u32 = TICK_HZ / 2;
/// Ticks from one rock to the next, somewhere in this range.
const GAP_LOW: u32 = 26;
const GAP_HIGH: u32 = 34;
pub(crate) const LASTS: u32 = 30 * TICK_HZ;
const FLOOR: Rgb = Rgb(52, 56, 72);
const ROCK: Rgb = Rgb(150, 132, 112);

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Square {
    Clear,
    /// A rock on its way down, landing when this many ticks have gone.
    Falling(u32),
    /// A landed rock crumbling, gone when this many ticks have gone.
    Landed(u32),
}

#[derive(Clone)]
pub(crate) struct Rocks {
    rng: Rng,
    pub(crate) at: (f32, f32),
    pub(crate) squares: [Square; SIDE * SIDE],
    pub(crate) tick: u32,
    next: u32,
}

/// Which square a point on the floor is in.
pub(crate) fn square_of(at: (f32, f32)) -> usize {
    let (column, row) = BOARD.nearest(at);
    row * SIDE + column
}

fn corner(square: usize) -> (f32, f32) {
    BOARD.corner(square % SIDE, square / SIDE)
}

impl Rocks {
    pub(crate) fn new(seed: u64) -> Self {
        Rocks {
            rng: Rng::new(seed),
            at: (BOARD.left + REGION * 1.5, BOARD.top + REGION * 1.5),
            squares: [Square::Clear; SIDE * SIDE],
            tick: 0,
            next: GAP_HIGH,
        }
    }

    /// A third of the rocks go for the square the player is standing in, and
    /// the rest for any square that has nothing already falling on it.
    fn drop(&mut self) {
        let free: Vec<usize> = (0..SIDE * SIDE)
            .filter(|square| !matches!(self.squares[*square], Square::Falling(_)))
            .collect();
        let under = square_of(self.at);
        let square = if free.contains(&under) && self.rng.chance(1.0 / 3.0) {
            under
        } else {
            free[self.rng.below(free.len())]
        };
        self.squares[square] = Square::Falling(FALL);
    }
}

impl Game for Rocks {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some((across, down)) = input.heading() {
            let far = SIDE as f32 * REGION - DOT;
            self.at.0 =
                (self.at.0 + across * PACE * TICK).clamp(BOARD.left + DOT, BOARD.left + far);
            self.at.1 = (self.at.1 + down * PACE * TICK).clamp(BOARD.top + DOT, BOARD.top + far);
        }

        self.tick += 1;
        let under = square_of(self.at);
        let mut crushed = false;
        for (square, state) in self.squares.iter_mut().enumerate() {
            *state = match *state {
                Square::Falling(1) => {
                    crushed |= square == under;
                    Square::Landed(CRUMBLE)
                }
                Square::Falling(left) => Square::Falling(left - 1),
                Square::Landed(1) | Square::Clear => Square::Clear,
                Square::Landed(left) => Square::Landed(left - 1),
            };
        }
        if crushed {
            return Outcome::Lost;
        }
        if self.tick >= LASTS {
            return Outcome::Won;
        }
        if self.tick >= self.next {
            self.drop();
            self.next =
                self.tick + GAP_LOW + self.rng.below((GAP_HIGH - GAP_LOW + 1) as usize) as u32;
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let (across, down) = BOARD.span();
        frame.rect(BOARD.left, BOARD.top, across, down, FLOOR);
        for line in 0..=SIDE {
            let along = line as f32 * REGION;
            frame.rect(BOARD.left + along - 1.0, BOARD.top, 2.0, down, palette::EDGE);
            frame.rect(BOARD.left, BOARD.top + along - 1.0, across, 2.0, palette::EDGE);
        }

        for (square, state) in self.squares.iter().enumerate() {
            let (x, y) = corner(square);
            match *state {
                Square::Falling(left) => {
                    let near = 1.0 - left as f32 / FALL as f32;
                    let shade = 0.3 * REGION * (1.0 - near);
                    let size = REGION - 2.0 * shade;
                    frame.rect(
                        x + shade,
                        y + shade,
                        size,
                        size,
                        FLOOR.mix(palette::BACKDROP, 0.25 + 0.6 * near),
                    );
                }
                Square::Landed(left) => {
                    let gone = 1.0 - left as f32 / CRUMBLE as f32;
                    let inset = 6.0 + 30.0 * gone;
                    let size = REGION - 2.0 * inset;
                    frame.rect(x + inset, y + inset, size, size, ROCK.mix(FLOOR, gone));
                }
                Square::Clear => {}
            }
        }

        let mut falling: Vec<(usize, u32)> = self
            .squares
            .iter()
            .enumerate()
            .filter_map(|(square, state)| match state {
                Square::Falling(left) => Some((square, *left)),
                _ => None,
            })
            .collect();
        falling.sort_by_key(|(_, left)| *left);
        for (square, left) in falling {
            let (x, y) = corner(square);
            let near = 1.0 - left as f32 / FALL as f32;
            let grow = 0.45 * REGION * (1.0 - near);
            let size = REGION + 2.0 * grow - 12.0;
            let (rx, ry) = (x - grow + 6.0, y - grow + 6.0);
            frame.outline(rx, ry, size, size, 2.0 + 3.0 * near, FLOOR.mix(ROCK, 0.2 + 0.8 * near));
            frame.outline(x + 1.0, y + 1.0, REGION - 2.0, REGION - 2.0, 3.0, palette::WARN);
        }

        frame.disc(self.at.0, self.at.1, DOT, palette::PLAYER);
    }
}

#[cfg(test)]
#[path = "../tests/games/rocks.rs"]
mod tests;
