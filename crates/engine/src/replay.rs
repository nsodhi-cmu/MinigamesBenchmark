//! An episode played again from its seed and everything that landed on it, one
//! screen for every tick of it.

use crate::frame::Frame;
use crate::game::Outcome;
use crate::input::Action;
use crate::session::Session;

/// Plays `session`, fresh from its game and seed, through `record` until `end`
/// ticks have passed or it is over, handing `each` the screen as it starts and
/// after every tick. The session it ends on is the answer, for whoever played the
/// original to compare with; an error from `each` stops it.
pub fn replay<E>(
    mut session: Session,
    record: &[(u32, Action)],
    end: u32,
    mut each: impl FnMut(&Frame) -> Result<(), E>,
) -> Result<Session, E> {
    let mut screen = Frame::new();
    session.draw(&mut screen);
    each(&screen)?;
    let mut landed = record.iter().peekable();
    while session.tick() < end && session.outcome() == Outcome::Playing {
        while let Some((_, action)) = landed.next_if(|(tick, _)| *tick <= session.tick()) {
            session.apply(*action);
        }
        session.step();
        session.draw(&mut screen);
        each(&screen)?;
    }
    let last = session.tick();
    for (_, action) in landed.take_while(|(tick, _)| *tick <= last) {
        session.apply(*action);
    }
    Ok(session)
}

#[cfg(test)]
#[path = "tests/replay.rs"]
mod tests;
