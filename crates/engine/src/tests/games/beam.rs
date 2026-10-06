use super::{Beam, Ends, ROOMS_LAID, Room, trace};
use crate::game::{Game, Outcome};
use crate::input::{Action, Input, Key};
use crate::playing::LOOK;

/// How many evenly spaced angles of the mirror the searches below try.
const STEPS: usize = 3600;

/// The angle of the mirror at one of the `STEPS`.
fn angle(step: usize) -> f32 {
    step as f32 * std::f32::consts::TAU / STEPS as f32
}

/// An angle of the mirror that sends the beam home in this room.
fn winning_turn(room: usize) -> Option<f32> {
    (0..STEPS).map(angle).find(|turn| trace(&ROOMS_LAID[room], *turn).ends == Ends::Portal)
}

/// The widest unbroken run of angles of the mirror that send the beam home, in
/// degrees.
fn widest_window(room: &Room) -> f32 {
    let mut home = vec![false; STEPS];
    for (step, hit) in home.iter_mut().enumerate() {
        *hit = trace(room, angle(step)).ends == Ends::Portal;
    }
    let mut widest = 0;
    let mut run = 0;
    for step in 0..STEPS * 2 {
        if home[step % STEPS] {
            run += 1;
            widest = widest.max(run);
        } else {
            run = 0;
        }
    }
    widest as f32 * 360.0 / STEPS as f32
}

/// The beam always reaches the mirror before anything else, so the room the
/// player is shown is the room they are playing.
#[test]
fn the_first_leg_of_every_beam_lands_on_the_mirror() {
    for (number, room) in ROOMS_LAID.iter().enumerate() {
        for step in 0..360 {
            let turn = step as f32 * std::f32::consts::TAU / 360.0;
            let shot = trace(room, turn);
            let bend = shot.bends[1];
            let off = libm::hypotf(bend.0 - room.mirror[0], bend.1 - room.mirror[1]);
            assert!(off <= 31.0, "room {} misses its own mirror by {off:.1}", number + 1);
        }
    }
}

/// Turning the mirror to the middle of a winning window and sending the beam
/// clears every room, one after another, which is what makes the five of them
/// finishable by a player who never sees inside them.
#[test]
fn sending_the_beam_at_a_winning_angle_clears_all_five_rooms() {
    let mut game = Beam::new();
    let mut input = Input::new();
    let mut outcome = Outcome::Playing;
    for (number, room) in ROOMS_LAID.iter().enumerate() {
        let good = middle_of_a_window(room);
        // Sweep toward the angle while it is a long way off and tap the
        // arrow once a look when it is near, which is how it would be played.
        let rising = good > game.turn;
        let key = if rising { Key::Right } else { Key::Left };
        let wins = |turn: f32| trace(&ROOMS_LAID[number], turn).ends == Ends::Portal;
        for _ in 0..4000 {
            if wins(game.turn) {
                break;
            }
            let near = (game.turn - good).abs() < 0.16;
            input.clear();
            input.apply(Action::Down(key));
            for tick in 0..LOOK {
                if near && tick > 0 {
                    input.apply(Action::Up(key));
                }
                assert_eq!(game.update(&input), Outcome::Playing, "turning the mirror ended it");
                input.settle();
            }
        }
        input.clear();
        assert!(wins(game.turn), "room {} could not be laid on a winning angle", number + 1);
        input.apply(Action::Tap(Key::Space));
        for _ in 0..600 {
            outcome = game.update(&input);
            input.settle();
            assert_ne!(outcome, Outcome::Lost, "room {} lost at a winning angle", number + 1);
            if outcome == Outcome::Won || game.run.is_none() {
                break;
            }
        }
    }
    assert_eq!(outcome, Outcome::Won, "the five rooms were not finished");
}

/// The angle in the middle of the widest run of winning angles, which is the
/// one a player who has worked the room out would lay the mirror at.
fn middle_of_a_window(room: &Room) -> f32 {
    let winning = |step: usize| trace(room, angle(step)).ends == Ends::Portal;
    let (mut best, mut widest, mut run) = (0usize, 0usize, 0usize);
    for step in 0..STEPS * 2 {
        run = if winning(step % STEPS) { run + 1 } else { 0 };
        if run > widest {
            widest = run;
            best = step;
        }
    }
    let turn = angle((best + STEPS - widest / 2) % STEPS);
    assert!(
        trace(room, turn).ends == Ends::Portal,
        "the middle of a window is not a winning angle"
    );
    turn
}

/// A beam that finds only a wall costs the time it took, and one that finds a
/// post ends the round. That difference is the whole of the danger.
#[test]
fn a_wall_forgives_the_beam_and_a_post_does_not() {
    let fire = |room: usize, turn: f32| {
        let mut game = Beam::new();
        game.room = room;
        game.turn = turn;
        let mut input = Input::new();
        input.apply(Action::Tap(Key::Space));
        let mut outcome = Outcome::Playing;
        for _ in 0..600 {
            outcome = game.update(&input);
            input.settle();
            if outcome != Outcome::Playing || game.run.is_none() {
                break;
            }
        }
        (outcome, game.room)
    };

    for (number, room) in ROOMS_LAID.iter().enumerate() {
        for want in [Ends::Wall, Ends::Post] {
            let Some(turn) = (0..STEPS).map(angle).find(|t| trace(room, *t).ends == want) else {
                continue;
            };
            let (outcome, landed) = fire(number, turn);
            match want {
                Ends::Wall => {
                    assert_eq!(outcome, Outcome::Playing, "a wall ended room {}", number + 1);
                    assert_eq!(landed, number, "a wall moved room {} on", number + 1);
                }
                _ => assert_eq!(outcome, Outcome::Lost, "a post forgave in room {}", number + 1),
            }
        }
    }
}

/// A look of holding an arrow, a tenth of a second, has to turn the mirror by
/// less than the narrowest room's window, or a player who looks ten times a
/// second could step straight over a room's answer and never land on it.
#[test]
fn one_look_of_an_arrow_turns_the_mirror_less_than_the_narrowest_window() {
    let mut game = Beam::new();
    let was = game.turn;
    let mut input = Input::new();
    input.apply(Action::Down(Key::Right));
    for _ in 0..LOOK {
        game.update(&input);
        input.settle();
    }
    let step = (game.turn - was).to_degrees();
    let narrowest = ROOMS_LAID.iter().map(widest_window).fold(f32::MAX, f32::min);
    assert!(
        step < narrowest,
        "one look turns the mirror {step:.2} degrees and the narrowest window is {narrowest:.2}"
    );
}

/// The beam that lands is the beam that left. The light is off the mirror as
/// soon as it is sent, so turning the mirror after that cannot bend it.
#[test]
fn holding_an_arrow_through_a_shot_does_not_bend_the_beam_that_is_already_running() {
    let winning = winning_turn(0).expect("room one can be solved");
    let ends = |held: Option<Key>| {
        let mut beam = Beam::new();
        beam.turn = winning;
        let mut input = Input::new();
        input.apply(Action::Tap(Key::Space));
        if let Some(key) = held {
            input.apply(Action::Down(key));
        }
        let mut outcome = beam.update(&input);
        input.settle();
        let mut ticks = 0;
        while outcome == Outcome::Playing && beam.run.is_some() && ticks < 600 {
            outcome = beam.update(&input);
            ticks += 1;
        }
        (outcome, beam.room)
    };
    let alone = ends(None);
    assert_eq!(ends(Some(Key::Left)), alone, "holding LEFT changed where the beam landed");
    assert_eq!(ends(Some(Key::Right)), alone, "holding RIGHT changed where the beam landed");
}
