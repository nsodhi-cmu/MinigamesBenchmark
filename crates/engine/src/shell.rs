//! The screens on either side of an episode, and the way a live player is taken
//! through them: the menu, a brief, the round, and how it went.

use crate::catalog::CATALOG;
use crate::frame::{self, Frame, text_height, text_width};
use crate::game::{LIMIT, Outcome, Scoring, Spec};
use crate::input::{self, Action, Button, Key};
use crate::palette;
use crate::session::{Report, Session};

const TITLE: &str = "Minigames Benchmark";
/// The most ticks one slow frame is let catch up on.
const CATCH_UP: f32 = 6.0;

const TOP: f32 = 74.0;
const PITCH: f32 = 28.0;
const VISIBLE: usize = 12;
const LEFT: f32 = 72.0;
const SPAN: f32 = 480.0;
/// How many characters of tags fit beside the place counter.
const TAG_COLUMNS: usize = 46;

/// `text` broken into lines of at most `columns` characters, on word boundaries.
pub fn wrap(text: &str, columns: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > columns {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        lines.push(line);
    }
    lines
}

/// Every game there is, with one of them picked out. More games than fit are
/// scrolled to keep the picked one in view.
pub fn menu(frame: &mut Frame, chosen: usize) {
    let middle = frame::WIDTH as f32 * 0.5;
    frame.fill(palette::BACKDROP);
    frame.text_center(middle, 20.0, "MINIGAMES", 3, palette::INK);
    frame.rect(middle - 132.0, 56.0, 264.0, 2.0, palette::PLAYER);

    let total = CATALOG.len();
    let chosen = chosen.min(total - 1);
    let shown = VISIBLE.min(total);
    let first = chosen.saturating_sub(shown / 2).min(total - shown);

    for slot in 0..shown {
        let index = first + slot;
        let spec = &CATALOG[index].spec;
        let y = TOP + slot as f32 * PITCH;
        let picked = index == chosen;
        if picked {
            frame.rect(LEFT, y - 6.0, SPAN, PITCH - 2.0, palette::PANEL);
            frame.rect(LEFT, y - 6.0, 4.0, PITCH - 2.0, palette::PLAYER);
        }
        let ink = if picked { palette::INK } else { palette::MUTED };
        frame.text(LEFT + 22.0, y, spec.title, 2, ink);
    }

    if total > shown {
        let tall = shown as f32 * PITCH - 2.0;
        let thumb = tall * shown as f32 / total as f32;
        let travel = (tall - thumb) * first as f32 / (total - shown) as f32;
        frame.rect(LEFT + SPAN + 10.0, TOP - 6.0, 6.0, tall, palette::PANEL);
        frame.rect(LEFT + SPAN + 10.0, TOP - 6.0 + travel, 6.0, thumb, palette::PLAYER);
    }

    // The tags of whichever game is picked out, along the strip under the list,
    // on a second line above the first when one will not hold them.
    let mut lines: Vec<String> = vec![String::new()];
    for tag in CATALOG[chosen].spec.tags.iter().map(|tag| tag.name()) {
        let line = lines.last_mut().expect("one line to start with");
        if !line.is_empty() && line.len() + 3 + tag.len() > TAG_COLUMNS {
            lines.push(tag.to_string());
        } else {
            if !line.is_empty() {
                line.push_str("   ");
            }
            line.push_str(tag);
        }
    }
    for (row, line) in lines.iter().enumerate() {
        let y = 420.0 - (lines.len() - 1 - row) as f32 * (text_height(1) + 6.0);
        frame.text(LEFT + SPAN - text_width(line, 1), y, line, 1, palette::WARN);
    }
    let place = format!("{} of {}", chosen + 1, total);
    frame.text(LEFT, 420.0, &place, 1, palette::MUTED);
    frame.text_center(
        middle,
        448.0,
        "UP DOWN CHOOSE    ENTER PLAYS    ESC QUITS",
        1,
        palette::MUTED,
    );
}

pub fn brief(frame: &mut Frame, spec: &Spec, seed: u64) {
    let middle = frame::WIDTH as f32 * 0.5;
    frame.fill(palette::BACKDROP);
    frame.text_center(middle, 48.0, spec.title, 4, palette::INK);
    frame.rect(middle - 120.0, 92.0, 240.0, 2.0, palette::PLAYER);

    let mut y = 124.0;
    for line in wrap(spec.brief, 66) {
        frame.text_center(middle, y, &line, 1, palette::INK);
        y += text_height(1) + 6.0;
    }

    y += 12.0;
    for control in spec.controls {
        frame.text(96.0, y, control, 1, palette::MUTED);
        y += text_height(1) + 6.0;
    }

    let terms = match spec.scoring {
        Scoring::Boolean => format!("{LIMIT:.0}s limit  ~  pass or fail"),
        Scoring::Time => format!("{LIMIT:.0}s limit  ~  finish sooner to do better"),
    };
    frame.text_center(middle, 392.0, &terms, 1, palette::MUTED);
    if spec.varies {
        frame.text_center(middle, 410.0, &format!("seed {seed}"), 1, palette::EDGE);
    }
    frame.text_center(middle, 436.0, "PRESS ENTER TO BEGIN", 2, palette::WARN);
}

pub fn result(frame: &mut Frame, spec: &Spec, report: &Report) {
    let middle = frame::WIDTH as f32 * 0.5;
    frame.fill(palette::BACKDROP);

    let (word, tint) = match report.outcome {
        Outcome::Won => ("FINISHED", palette::GOOD),
        _ => ("FAILED", palette::BAD),
    };
    frame.text_center(middle, 130.0, spec.title, 2, palette::MUTED);
    frame.text_center(middle, 176.0, word, 5, tint);

    let detail = format!("{:.2}s", report.seconds);
    frame.text_center(middle, 256.0, &detail, 2, palette::INK);
    frame.text_center(middle, 420.0, "ENTER PLAYS AGAIN    ESC GOES BACK", 1, palette::MUTED);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Menu,
    Brief,
    Playing,
    Over,
}

/// A person's way through the games. Opened on the menu, escape comes back to
/// it; opened on one game, escape closes it. It reads no clock and knows no
/// device: it is handed actions, and told how much of the player's time passed.
pub struct Shell {
    chosen: usize,
    rooted: bool,
    seed: u64,
    rate: f32,
    showing: bool,
    session: Session,
    stage: Stage,
    owed: f32,
    // The key that changed the screen must be let go of before enter or escape
    // does anything else.
    settling: bool,
    open: bool,
    held: [bool; Key::COUNT],
    buttons: [bool; 2],
    cursor: Option<(f32, f32)>,
}

impl Shell {
    /// `rate` is ticks of game time per second told to `advance`, and
    /// `showing` whether the limit counts down on screen. None if no game
    /// has that name.
    pub fn new(game: Option<&str>, seed: u64, rate: f32, showing: bool) -> Option<Shell> {
        let chosen = match game {
            Some(name) => CATALOG.iter().position(|entry| entry.spec.name == name)?,
            None => 0,
        };
        Some(Shell {
            chosen,
            rooted: game.is_none(),
            seed,
            rate: rate.max(1.0),
            showing,
            session: start(chosen, seed, showing),
            stage: if game.is_none() { Stage::Menu } else { Stage::Brief },
            owed: 0.0,
            settling: false,
            open: true,
            held: [false; Key::COUNT],
            buttons: [false; 2],
            cursor: None,
        })
    }

    /// Whether there is still something on screen.
    pub fn open(&self) -> bool {
        self.open
    }

    /// What the window is called: the menu's name, or the game's beside it.
    pub fn title(&self) -> String {
        match self.stage {
            Stage::Menu => TITLE.to_string(),
            _ => format!("{TITLE} — {}", CATALOG[self.chosen].spec.title),
        }
    }

    /// The game on screen.
    pub fn spec(&self) -> &'static Spec {
        self.session.spec()
    }

    /// The seed that laid out the round on screen.
    pub fn seed(&self) -> u64 {
        self.session.seed()
    }

    /// One thing the player did. During a round the game is handed it too. A
    /// key that goes down again while held is a repeat, which only the menu
    /// listens to.
    pub fn act(&mut self, action: Action) {
        if self.stage == Stage::Playing {
            self.session.apply(action);
        }
        match action {
            Action::Move { x, y } | Action::Click { x, y, .. } => self.cursor = Some((x, y)),
            Action::Press { button, x, y } => {
                self.cursor = Some((x, y));
                self.buttons[button as usize] = true;
            }
            Action::Release { button, x, y } => {
                self.cursor = Some((x, y));
                self.buttons[button as usize] = false;
            }
            Action::Down(key) => {
                let fresh = !self.held[key as usize];
                self.settle();
                self.held[key as usize] = true;
                self.press(key, fresh);
            }
            Action::Up(key) => {
                self.held[key as usize] = false;
                self.settle();
            }
            Action::Tap(_) | Action::Wait => {}
        }
    }

    /// So much of the player's time passing. During a round that is game time
    /// at the rate, and the report comes back on the call where the round ends.
    pub fn advance(&mut self, seconds: f32) -> Option<Report> {
        if self.stage != Stage::Playing {
            return None;
        }
        self.owed = (self.owed + seconds * self.rate).min(CATCH_UP);
        while self.owed >= 1.0 {
            self.session.step();
            self.owed -= 1.0;
        }
        if self.session.outcome() == Outcome::Playing {
            return None;
        }
        self.seed = self.seed.wrapping_add(1);
        self.go(Stage::Over);
        Some(self.session.report())
    }

    pub fn draw(&self, frame: &mut Frame) {
        let spec = &CATALOG[self.chosen].spec;
        match self.stage {
            Stage::Menu => menu(frame, self.chosen),
            Stage::Brief => brief(frame, spec, self.seed),
            Stage::Playing => self.session.draw(frame),
            Stage::Over => result(frame, spec, &self.session.report()),
        }
    }

    fn settle(&mut self) {
        if !self.held[Key::Enter as usize] && !self.held[Key::Escape as usize] {
            self.settling = false;
        }
    }

    fn press(&mut self, key: Key, fresh: bool) {
        let total = CATALOG.len();
        match (self.stage, key) {
            (Stage::Menu, Key::Down) => self.chosen = (self.chosen + 1) % total,
            (Stage::Menu, Key::Up) => self.chosen = (self.chosen + total - 1) % total,
            _ if !fresh || self.settling => {}
            (Stage::Menu | Stage::Over, Key::Enter) => {
                self.session = start(self.chosen, self.seed, self.showing);
                self.go(Stage::Brief);
            }
            (Stage::Brief, Key::Enter) => self.begin(),
            (Stage::Menu, Key::Escape) => self.open = false,
            (_, Key::Escape) if self.rooted => self.go(Stage::Menu),
            (_, Key::Escape) => self.open = false,
            _ => {}
        }
    }

    /// Whatever is already held goes into the round with the player, except
    /// the enter that opened it.
    fn begin(&mut self) {
        self.owed = 0.0;
        for (key, _) in input::KEYS {
            if key != Key::Enter && self.held[key as usize] {
                self.session.apply(Action::Down(key));
            }
        }
        if let Some((x, y)) = self.cursor {
            self.session.apply(Action::Move { x, y });
            for button in [Button::Left, Button::Right] {
                if self.buttons[button as usize] {
                    self.session.apply(Action::Press { button, x, y });
                }
            }
        }
        self.go(Stage::Playing);
    }

    fn go(&mut self, stage: Stage) {
        self.stage = stage;
        self.settling = true;
    }
}

fn start(chosen: usize, seed: u64, showing: bool) -> Session {
    let mut session =
        Session::new(CATALOG[chosen].spec.name, seed).expect("the catalog names itself");
    session.show_clock(showing);
    session
}

#[cfg(test)]
#[path = "tests/shell.rs"]
mod tests;
