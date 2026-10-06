//! One episode of one game: the clock, the screen, and the result.

use crate::catalog;
use crate::frame::{self, Frame, text_width};
use crate::game::{Game, LIMIT, Outcome, Scoring, Spec, TICK_HZ};
use crate::input::{Action, Button, Input};
use crate::palette;
use std::num::NonZeroU32;

/// The outline of the pointer, from its tip.
const ARROW: [[f32; 2]; 7] =
    [[0.0, 0.0], [0.0, 14.0], [4.0, 11.0], [7.0, 17.0], [10.0, 16.0], [7.0, 10.0], [11.0, 10.0]];

pub struct Report {
    pub game: &'static str,
    pub outcome: Outcome,
    pub seconds: f32,
    pub scoring: Scoring,
}

pub struct Session {
    spec: &'static Spec,
    game: Box<dyn Game>,
    input: Input,
    seed: u64,
    tick: u32,
    limit_ticks: u32,
    clock: bool,
    pointed: bool,
    outcome: Outcome,
    record: Vec<(u32, Action)>,
}

impl Session {
    pub fn new(name: &str, seed: u64) -> Option<Session> {
        let entry = catalog::find(name)?;
        Some(Session {
            spec: &entry.spec,
            game: (entry.build)(seed),
            input: Input::new(),
            seed,
            tick: 0,
            limit_ticks: (LIMIT * TICK_HZ as f32) as u32,
            clock: false,
            pointed: false,
            outcome: Outcome::Playing,
            record: Vec::new(),
        })
    }

    /// Whether the clock counts down where it can be seen. It is hidden unless
    /// asked for: the limit still runs, and the time taken is told at the end.
    pub fn show_clock(&mut self, shown: bool) {
        self.clock = shown;
    }

    pub fn spec(&self) -> &'static Spec {
        self.spec
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn seconds(&self) -> f32 {
        self.tick as f32 / TICK_HZ as f32
    }

    /// The actions land, then `ticks` of game time pass.
    pub fn advance(&mut self, actions: &[Action], ticks: u32) -> Outcome {
        if self.outcome != Outcome::Playing {
            return self.outcome;
        }
        for action in actions {
            self.apply(*action);
        }
        self.lapse(ticks);
        self.outcome
    }

    /// Game time passing with whatever is held still held.
    pub fn lapse(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    /// One tick of game time, for a player driving the game live.
    pub fn step(&mut self) {
        if self.outcome != Outcome::Playing {
            return;
        }
        self.outcome = self.game.update(&self.input);
        self.input.settle();
        self.tick += 1;
        if self.outcome == Outcome::Playing && self.tick >= self.limit_ticks {
            self.outcome = Outcome::Lost;
            self.input.clear();
        }
    }

    /// One thing the player did, landing at the start of the tick to come. Once the
    /// episode is over nothing lands.
    pub fn apply(&mut self, action: Action) {
        if self.outcome != Outcome::Playing {
            return;
        }
        self.record.push((self.tick, action));
        self.pointed |= matches!(
            action,
            Action::Move { .. }
                | Action::Press { .. }
                | Action::Release { .. }
                | Action::Click { .. }
        );
        self.input.apply(action);
    }

    /// Everything that landed on the episode, each beside the tick it landed at the
    /// start of, in the order it landed: all a replay needs besides the game and seed.
    pub fn record(&self) -> &[(u32, Action)] {
        &self.record
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.fill(palette::BACKDROP);
        self.game.draw(frame);
        if self.clock {
            self.draw_clock(frame);
        }
        if self.spec.pointer && self.pointed {
            self.draw_cursor(frame);
        }
    }

    /// The pointer, drawn into the picture once something has put it somewhere: an
    /// open arrow whose tip is the point, so what it points at stays readable.
    fn draw_cursor(&self, frame: &mut Frame) {
        let (x, y) = self.input.cursor();
        let held = self.input.button_held(Button::Left);
        let tint = if held { palette::WARN } else { palette::INK };
        let outline = ARROW.iter().zip(ARROW.iter().cycle().skip(1));
        for (weight, color) in [(2.0, palette::BACKDROP), (1.0, tint)] {
            for (a, b) in outline.clone() {
                frame.line(x + a[0], y + a[1], x + b[0], y + b[1], weight, color);
            }
        }
    }

    /// What is left of the limit, in minutes, seconds, and frames, across the top
    /// of the picture.
    fn draw_clock(&self, frame: &mut Frame) {
        let left = self.limit_ticks.saturating_sub(self.tick);
        let clock =
            format!("{}:{:02}:{:02}", left / (60 * TICK_HZ), (left / TICK_HZ) % 60, left % TICK_HZ);
        let tint = if left <= 5 * TICK_HZ { palette::BAD } else { palette::INK };
        frame.text(frame::WIDTH as f32 * 0.5 - text_width(&clock, 2) * 0.5, 6.0, &clock, 2, tint);
    }

    pub fn report(&self) -> Report {
        Report {
            game: self.spec.name,
            outcome: self.outcome,
            seconds: self.seconds(),
            scoring: self.spec.scoring,
        }
    }
}

/// How game time passes for an agent playing on a screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pace {
    /// Ticks of game time to a second of the agent's.
    Clock(NonZeroU32),
    /// The game stands still, and each press of `ADVANCE` lets this many ticks pass.
    Ticks(NonZeroU32),
}

/// The key that lets time pass when the pace is in ticks. No game reads it.
pub const ADVANCE: &str = "F1";

/// What is true of every game when an agent plays it on a screen with a mouse
/// and a keyboard, at `pace`.
pub fn desktop(pace: Pace) -> String {
    let screen = format!(
        "You are playing a small game on a screen {width} by {height} pixels. The game fills the \
         screen and nothing else is on it: no desktop, no other window, and nothing to open. \
         Everything you need to play is drawn on the screen, and you play with the mouse and the \
         keyboard. Where the game uses the pointer, it draws the pointer itself, as a small open \
         arrow, once the mouse has moved.",
        width = frame::WIDTH,
        height = frame::HEIGHT,
    );
    let attempt = "You have one attempt, and it ends on its own when the game is won or lost or \
                   its time runs out. You will not be told how it went and nobody will answer a \
                   question, so keep playing until you are stopped.";
    match pace {
        Pace::Clock(rate) => {
            let rate = rate.get();
            let speed = if rate == TICK_HZ {
                "The game runs at its normal speed.".to_string()
            } else {
                let slower = TICK_HZ as f32 / rate as f32;
                let limit = LIMIT * slower;
                let limit = if limit >= 120.0 {
                    format!("{} minutes", trimmed(limit / 60.0))
                } else {
                    format!("{} seconds", trimmed(limit))
                };
                format!(
                    "The game runs {slower} times slower than normal. Every second of it takes \
                     {slower} seconds of yours, so its time limit of {LIMIT:.0} seconds is {limit} \
                     of yours.",
                    slower = trimmed(slower),
                )
            };
            format!(
                "{screen}\n\nThe game started the moment you did. {attempt}\n\n{speed} The game \
                 reads the mouse and the keyboard once a frame, {rate} frames to a second of \
                 yours, and whatever happens within one frame lands together: a key pressed and \
                 let go within one frame counts once, and keys typed within one frame land in no \
                 particular order."
            )
        }
        Pace::Ticks(ticks) => format!(
            "{screen}\n\nThe game stands still until you press {ADVANCE}. Each press of \
             {ADVANCE} lets {ticks} {frames} pass, a frame being a sixtieth of a second of game \
             time, and then the game stands still again until the next press, so its time limit \
             of {LIMIT:.0} seconds is {presses} presses of {ADVANCE}. The new frame is drawn a \
             moment after each press. {attempt}\n\n\
             At each press of {ADVANCE} the game reads what is held down and where the pointer \
             is, all of it together, and the pointer it draws moves only then. A key or button \
             pressed and let go between two presses of {ADVANCE} does nothing. To press \
             something, hold it down, press {ADVANCE}, then let it go and press {ADVANCE} again; \
             to hold it for longer, press {ADVANCE} more times before letting go. To drag, hold \
             the button down, press {ADVANCE}, move the pointer, press {ADVANCE}, then let go and \
             press {ADVANCE}. Waiting does not move the game; only {ADVANCE} does.",
            frames = if ticks.get() == 1 { "frame" } else { "frames" },
            presses = (LIMIT * TICK_HZ as f32 / ticks.get() as f32).ceil(),
        ),
    }
}

/// Everything an agent is told before it plays `spec` at `pace`: the screen and
/// the pace, then the game.
pub fn task(spec: &Spec, pace: Pace) -> String {
    format!("{}\n\n{}", desktop(pace), spec.prompt())
}

/// A number with at most two places, and none where it is whole.
fn trimmed(number: f32) -> String {
    format!("{number:.2}").trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
#[path = "tests/session.rs"]
mod tests;
