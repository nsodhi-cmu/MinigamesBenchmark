//! What a game is, and what it is worth.

use crate::frame::Frame;
use crate::input::Input;

/// Ticks of game time in one second of game time. Every game is written to it.
pub const TICK_HZ: u32 = 60;
pub const TICK: f32 = 1.0 / TICK_HZ as f32;
/// Seconds of game time before any episode is lost: a safety net, the same for every game.
pub const LIMIT: f32 = 180.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scoring {
    /// Finished inside the limit, or not.
    Boolean,
    /// Finished inside the limit, and how long it took.
    Time,
}

/// What a game asks of a player, written down so that a run can be read by the
/// kind of thing it was rather than one game at a time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tag {
    // How long it takes. Every game carries exactly one of these.
    Fixed,
    Bounded,
    Unbounded,
    // What can end it early. Every game carries exactly one of these.
    Lethal,
    Hostile,
    Strict,
    Friendly,
    // What it asks of you.
    Puzzle,
    Quiz,
    Language,
    Memory,
    Aim,
    Timing,
    Physics,
    Navigate,
    Explore,
    Overhead,
    Platformer,
    Grid,
    Drag,
    Slippery,
}

impl Tag {
    pub fn name(self) -> &'static str {
        match self {
            Tag::Fixed => "fixed",
            Tag::Bounded => "bounded",
            Tag::Unbounded => "unbounded",
            Tag::Lethal => "lethal",
            Tag::Hostile => "hostile",
            Tag::Strict => "strict",
            Tag::Friendly => "friendly",
            Tag::Puzzle => "puzzle",
            Tag::Quiz => "quiz",
            Tag::Language => "language",
            Tag::Memory => "memory",
            Tag::Aim => "aim",
            Tag::Timing => "timing",
            Tag::Physics => "physics",
            Tag::Navigate => "navigate",
            Tag::Explore => "explore",
            Tag::Overhead => "overhead",
            Tag::Platformer => "platformer",
            Tag::Grid => "grid",
            Tag::Drag => "drag",
            Tag::Slippery => "slippery",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Playing,
    Won,
    Lost,
}

impl Outcome {
    pub fn name(self) -> &'static str {
        match self {
            Outcome::Playing => "playing",
            Outcome::Won => "won",
            Outcome::Lost => "lost",
        }
    }
}

/// Everything about a game that does not change from one episode to the next.
pub struct Spec {
    pub name: &'static str,
    pub title: &'static str,
    pub brief: &'static str,
    pub controls: &'static [&'static str],
    /// Whether the pointer is part of the controls, and so drawn into the picture.
    pub pointer: bool,
    pub scoring: Scoring,
    /// Whether the seed decides anything. A game that lays itself out the same
    /// way every time is not worth showing a seed for.
    pub varies: bool,
    /// What kind of thing this game is.
    pub tags: &'static [Tag],
}

impl Spec {
    /// The whole task, as a player is told it before the episode starts.
    pub fn prompt(&self) -> String {
        let mut out = format!("{}\n\n{}\n\nControls:\n", self.title, self.brief);
        for control in self.controls {
            out.push_str(&format!("  {control}\n"));
        }
        out.push_str(&match self.scoring {
            Scoring::Boolean => {
                format!("\nFinish within {:.0} seconds. You either finish or you do not.", LIMIT)
            }
            Scoring::Time => {
                format!("\nFinish within {:.0} seconds. The less time you take, the better.", LIMIT)
            }
        });
        out
    }
}

pub trait Game {
    /// One tick of game time.
    fn update(&mut self, input: &Input) -> Outcome;

    fn draw(&self, frame: &mut Frame);
}
