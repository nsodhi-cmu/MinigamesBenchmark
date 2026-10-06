//! Small games that a person can finish in under a minute and an agent can be
//! asked to finish from pictures alone.

#[cfg(test)]
#[path = "tests/playing.rs"]
mod playing;

pub mod aim;
pub mod board;
pub mod catalog;
pub mod frame;
pub mod game;
pub mod games;
pub mod input;
pub mod map;
pub mod maze;
pub mod palette;
pub mod replay;
pub mod rng;
pub mod session;
pub mod shape;
pub mod shell;
pub mod sky;
pub mod walk;
pub mod words;

pub use frame::{Frame, HEIGHT, Rgb, WIDTH};
pub use game::{Game, LIMIT, Outcome, Scoring, Spec, TICK, TICK_HZ};
pub use input::{Action, Button, Input, Key};
pub use session::{Report, Session};
