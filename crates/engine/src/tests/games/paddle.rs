use super::{BALL, FASTEST, NUDGE, PADDLE_HALF, PADDLE_Y, Paddle, SURVIVE, WALL};
use crate::frame;
use crate::game::{Game, Outcome, TICK_HZ};
use crate::input::{Action, Input, Key};
use crate::playing::LOOK;

/// Where the ball will be by the time it is back down at the paddle, counting the
/// walls it comes off on the way.
fn landing(game: &Paddle) -> f32 {
    let (mut x, mut y) = game.ball;
    let (mut across, mut down) = game.speed;
    for _ in 0..600 {
        down += 1500.0 / TICK_HZ as f32;
        x += across / TICK_HZ as f32;
        y += down / TICK_HZ as f32;
        if x < WALL + BALL || x > frame::WIDTH as f32 - WALL - BALL {
            x = x.clamp(WALL + BALL, frame::WIDTH as f32 - WALL - BALL);
            across = -across;
        }
        if down > 0.0 && y >= PADDLE_Y - BALL {
            break;
        }
    }
    x
}

fn play(mut steer: impl FnMut(&Paddle) -> Option<Key>) -> (Outcome, u32) {
    let mut game = Paddle::new();
    for tick in 1..=SURVIVE {
        let mut input = Input::new();
        if let Some(key) = steer(&game) {
            input.apply(Action::Down(key));
        }
        let outcome = game.update(&input);
        if outcome != Outcome::Playing {
            return (outcome, tick);
        }
    }
    (Outcome::Playing, SURVIVE)
}

/// A player who puts the middle of the paddle where the ball is coming down keeps
/// it up for the whole half minute.
#[test]
fn a_player_who_gets_under_the_ball_keeps_it_up_for_the_whole_round() {
    let mut aim = 0.0f32;
    let mut look = 0;
    let (outcome, ticks) = play(|game| {
        if look == 0 {
            aim = landing(game);
        }
        look = (look + 1) % LOOK;
        let gap = aim - game.paddle;
        if gap.abs() < PADDLE_HALF * 0.3 {
            None
        } else if gap > 0.0 {
            Some(Key::Right)
        } else {
            Some(Key::Left)
        }
    });
    assert_eq!(outcome, Outcome::Won, "the ball was dropped after {ticks} ticks");
}

/// A tilted paddle nudges the ball a little its way, never past the limit, and
/// a flat one leaves the ball going the way it was.
#[test]
fn a_tilted_paddle_nudges_the_ball_its_way_up_to_a_limit_and_a_flat_one_does_not() {
    let cases = [
        (Some(Key::Right), -130.0, -130.0 + NUDGE),
        (Some(Key::Left), -130.0, -130.0 - NUDGE),
        (None, -130.0, -130.0),
        (Some(Key::Left), -FASTEST, -FASTEST),
    ];
    for (held, from, want) in cases {
        let mut game = Paddle::new();
        game.ball = (game.paddle, PADDLE_Y - BALL - 1.0);
        game.speed = (from, 300.0);
        let mut input = Input::new();
        if let Some(key) = held {
            input.apply(Action::Down(key));
        }
        game.update(&input);
        assert_eq!(
            game.speed.0, want,
            "holding {held:?} from {from} sent the ball at {}",
            game.speed.0
        );
        assert!(game.speed.1 < 0.0, "holding {held:?} did not bounce the ball");
    }
}
