//! Play the games by hand, look at what an agent would see, or list what there is.

mod screen;
mod window;

use clap::{Args, Parser, Subcommand};
use minigame::catalog::{self, CATALOG};
use minigame::game::{LIMIT, Scoring};
use minigame::session::{Pace, Session};
use std::num::NonZeroU32;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "minigame", about = "Small games for people and for agents that only see pixels")]
struct Cli {
    /// Left out, the menu opens.
    #[command(subcommand)]
    command: Option<Command>,
}

/// How game time passes for an agent: on a clock, or a few ticks at each press of the advance key.
#[derive(Args)]
struct Paced {
    /// Ticks of game time per second of the agent's, from 1 to 60. 60 unless `--ticks` is given.
    #[arg(long, conflicts_with = "ticks", value_parser = paced)]
    clock: Option<NonZeroU32>,
    /// Instead: the game stands still, and each press of the advance key lets this many ticks
    /// pass, from 1 to 60. 1 if given without a number.
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "1",
        value_parser = paced
    )]
    ticks: Option<NonZeroU32>,
}

/// The clock the games are written to.
const FULL_SPEED: NonZeroU32 = NonZeroU32::new(minigame::TICK_HZ).unwrap();

/// A count of ticks from 1 to 60, as `--clock` and `--ticks` take it.
fn paced(given: &str) -> Result<NonZeroU32, String> {
    given
        .parse()
        .ok()
        .filter(|count: &NonZeroU32| count.get() <= minigame::TICK_HZ)
        .ok_or_else(|| format!("{given} is not a whole number from 1 to {}", minigame::TICK_HZ))
}

impl Paced {
    fn pace(&self) -> Pace {
        match (self.clock, self.ticks) {
            (_, Some(ticks)) => Pace::Ticks(ticks),
            (clock, None) => Pace::Clock(clock.unwrap_or(FULL_SPEED)),
        }
    }
}

#[derive(Subcommand)]
enum Command {
    /// Every game there is.
    List,
    /// What a player is told before an episode starts.
    Prompt { game: String },
    /// What an agent playing on a screen is told: the screen and the pace, then the game.
    Task {
        game: String,
        #[command(flatten)]
        paced: Paced,
    },
    /// Play with a keyboard and a mouse. Naming no game opens the menu.
    Play {
        game: Option<String>,
        /// The layout to play. Left out, every round is a fresh one.
        #[arg(long)]
        seed: Option<u64>,
        /// Ticks of game time per second of yours. The games are written to 60.
        #[arg(long, default_value_t = minigame::TICK_HZ as f32)]
        clock: f32,
        /// Count the limit down where it can be seen. Off unless asked for.
        #[arg(long)]
        countdown: bool,
    },
    /// One round on a virtual display of its own, for an agent beside it. Run in the game image.
    Screen {
        game: String,
        /// The layout to play.
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[command(flatten)]
        paced: Paced,
    },
    /// Write one frame to a PNG, the way an agent would receive it.
    Shot {
        game: String,
        /// The layout to draw.
        #[arg(long, default_value_t = 0)]
        seed: u64,
        /// Ticks of game time to let pass first.
        #[arg(long, default_value_t = 0)]
        after: u32,
        /// Draw the instructions screen instead of the game.
        #[arg(long)]
        brief: bool,
        /// Count the limit down where it can be seen.
        #[arg(long)]
        countdown: bool,
        #[arg(long, default_value = "shot.png")]
        out: String,
    },
}

fn main() -> std::process::ExitCode {
    let result = match Cli::parse().command.unwrap_or(Command::Play {
        game: None,
        seed: None,
        clock: minigame::TICK_HZ as f32,
        countdown: false,
    }) {
        Command::List => {
            list();
            Ok(())
        }
        Command::Prompt { game } => prompt(&game),
        Command::Task { game, paced } => task(&game, paced.pace()),
        Command::Screen { game, seed, paced } => screen::play(&game, seed, paced.pace()),
        Command::Play { game, seed, clock, countdown } => {
            window::play(game.as_deref(), seed.unwrap_or_else(fresh_seed), clock, countdown)
        }
        Command::Shot { game, seed, after, brief, countdown, out } => {
            shot(&game, seed, after, brief, countdown, &out)
        }
    };
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn fresh_seed() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0)
}

fn list() {
    println!("{:<10} {:<24} {:>7}  SCORED ON", "NAME", "TITLE", "LIMIT");
    for entry in CATALOG {
        let spec = &entry.spec;
        let scoring = match spec.scoring {
            Scoring::Boolean => "finishing",
            Scoring::Time => "time",
        };
        println!("{:<10} {:<24} {:>6.0}s  {}", spec.name, spec.title, LIMIT, scoring);
    }
}

fn prompt(game: &str) -> Result<(), String> {
    let entry = catalog::find(game).ok_or_else(|| unknown(game))?;
    println!("{}", entry.spec.prompt());
    Ok(())
}

fn task(game: &str, pace: Pace) -> Result<(), String> {
    let entry = catalog::find(game).ok_or_else(|| unknown(game))?;
    println!("{}", minigame::session::task(&entry.spec, pace));
    Ok(())
}

fn shot(
    game: &str,
    seed: u64,
    after: u32,
    brief: bool,
    countdown: bool,
    out: &str,
) -> Result<(), String> {
    let mut session = Session::new(game, seed).ok_or_else(|| unknown(game))?;
    session.show_clock(countdown);
    let mut frame = minigame::Frame::new();
    if brief {
        minigame::shell::brief(&mut frame, session.spec(), seed);
    } else {
        for _ in 0..after {
            session.step();
        }
        session.draw(&mut frame);
    }
    std::fs::write(out, frame.to_png()).map_err(|e| format!("could not write {out}: {e}"))?;
    println!("{out}");
    Ok(())
}

fn unknown(game: &str) -> String {
    format!(
        "no game called {game}. Known games: {}",
        catalog::names().collect::<Vec<_>>().join(", ")
    )
}
