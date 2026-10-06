use crate::catalog;
use crate::frame::Frame;
use crate::input::{Action, Key};
use crate::shell::{self, Shell};

const A_FRAME: f32 = 1.0 / 60.0;

fn picture(shell: &Shell) -> Frame {
    let mut frame = Frame::new();
    shell.draw(&mut frame);
    frame
}

/// The car in Raceway Royale turns while right is held, so the picture says
/// whether the game was handed the key.
#[test]
fn a_key_held_through_the_brief_is_held_when_the_round_starts() {
    let run = |before: bool| {
        let mut shell = Shell::new(Some("race"), 0, 60.0, false).expect("it is cataloged");
        if before {
            shell.act(Action::Down(Key::Right));
        }
        shell.act(Action::Down(Key::Enter));
        if !before {
            shell.act(Action::Down(Key::Right));
        }
        for _ in 0..60 {
            shell.advance(A_FRAME);
        }
        picture(&shell)
    };
    let mut idle = Shell::new(Some("race"), 0, 60.0, false).expect("it is cataloged");
    idle.act(Action::Down(Key::Enter));
    for _ in 0..60 {
        idle.advance(A_FRAME);
    }
    assert!(run(true).pixels() == run(false).pixels());
    assert!(run(true).pixels() != picture(&idle).pixels());
}

#[test]
fn enter_after_a_round_ends_does_nothing_until_it_is_let_go() {
    let mut shell = Shell::new(Some("serpent"), 0, 60.0, false).expect("it is cataloged");
    shell.act(Action::Down(Key::Enter));
    let report = std::iter::repeat_with(|| shell.advance(0.1)).find_map(|r| r).unwrap();
    let mut over = Frame::new();
    shell::result(&mut over, shell.spec(), &report);
    assert!(picture(&shell).pixels() == over.pixels());

    shell.act(Action::Down(Key::Enter));
    assert!(picture(&shell).pixels() == over.pixels(), "a repeat of the held enter");
    shell.act(Action::Up(Key::Enter));
    shell.act(Action::Down(Key::Enter));
    let mut brief = Frame::new();
    shell::brief(&mut brief, shell.spec(), shell.seed());
    assert!(picture(&shell).pixels() == brief.pixels(), "a fresh enter plays again");
    assert_eq!(shell.seed(), 1, "on the next layout");
}

#[test]
fn escape_closes_a_game_opened_by_name_and_goes_back_to_the_menu_otherwise() {
    let mut named = Shell::new(Some("serpent"), 0, 60.0, false).expect("it is cataloged");
    named.act(Action::Down(Key::Escape));
    assert!(!named.open());

    let mut rooted = Shell::new(None, 0, 60.0, false).expect("the menu always opens");
    rooted.act(Action::Down(Key::Enter));
    assert_eq!(rooted.title(), format!("Minigames Benchmark — {}", catalog::CATALOG[0].spec.title));
    rooted.act(Action::Down(Key::Escape));
    assert_ne!(
        rooted.title(),
        "Minigames Benchmark",
        "escape waits for the enter that opened the brief"
    );
    rooted.act(Action::Up(Key::Enter));
    rooted.act(Action::Up(Key::Escape));
    rooted.act(Action::Down(Key::Escape));
    assert_eq!(rooted.title(), "Minigames Benchmark");
    assert!(rooted.open());
}
