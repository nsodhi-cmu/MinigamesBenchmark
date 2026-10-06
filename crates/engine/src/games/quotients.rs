//! Five cards, four of a kind. Say which one is not.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, Tag};
use crate::input::{Button, Input};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "quotients",
    title: "Intel Seeker",
    brief: "Five cards are laid out. Four of them agree on everything that can be seen; one \
            of them differs, in its shape or its color or which way it faces or how many \
            it carries. Click the one that does not belong. Fifteen right answers finish the \
            round; one wrong one ends it.",
    controls: &["left click a card"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Unbounded, Tag::Strict, Tag::Quiz],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Quotients::new(seed))
}

const CARDS: usize = 5;
const ROUNDS: u32 = 15;
const WIDE: f32 = 112.0;
const TALL: f32 = 168.0;
const GAP: f32 = 14.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Shape {
    Triangle,
    Square,
    Round,
    Chevron,
}

impl Shape {
    /// Whether turning one shows. A circle turned is the same circle.
    #[cfg(test)]
    fn turns_visibly(self) -> bool {
        matches!(self, Shape::Triangle | Shape::Chevron)
    }
}

const SHAPES: [Shape; 4] = [Shape::Triangle, Shape::Square, Shape::Round, Shape::Chevron];
const TURNERS: [Shape; 2] = [Shape::Triangle, Shape::Chevron];
const TINTS: [Rgb; 5] =
    [palette::PLAYER, palette::GOOD, palette::WARN, palette::MARK, palette::PINK];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Card {
    pub shape: Shape,
    pub tint: usize,
    pub turn: usize,
    pub count: usize,
    /// Filler. It never picks a card out, but it is there to be looked at.
    pub pips: usize,
    pub tag: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Trait {
    Shape,
    Tint,
    Turn,
    Count,
    Pips,
    Tag,
}

/// The traits an answer may turn on.
const ANSWERABLE: [Trait; 4] = [Trait::Shape, Trait::Tint, Trait::Turn, Trait::Count];
/// Every trait a card carries, answerable or filler.
pub(crate) const TRAITS: [Trait; 6] =
    [Trait::Shape, Trait::Tint, Trait::Turn, Trait::Count, Trait::Pips, Trait::Tag];
const TAGS: usize = 5;
const PIPS: usize = 5;

fn span_of(which: Trait) -> usize {
    match which {
        Trait::Shape => SHAPES.len(),
        Trait::Tint => TINTS.len(),
        Trait::Turn => 4,
        Trait::Count => 3,
        Trait::Pips => PIPS,
        Trait::Tag => TAGS,
    }
}

fn set(card: &mut Card, which: Trait, value: usize) {
    match which {
        Trait::Shape => card.shape = SHAPES[value],
        Trait::Tint => card.tint = value,
        Trait::Turn => card.turn = value,
        Trait::Count => card.count = value + 1,
        Trait::Pips => card.pips = value,
        Trait::Tag => card.tag = value,
    }
}

#[cfg(test)]
pub(crate) fn read(card: &Card, which: Trait) -> usize {
    match which {
        Trait::Shape => SHAPES.iter().position(|s| *s == card.shape).expect("a known shape"),
        Trait::Tint => card.tint,
        Trait::Turn => card.turn,
        Trait::Count => card.count - 1,
        Trait::Pips => card.pips,
        Trait::Tag => card.tag,
    }
}

/// Two different values under `count`, in a stable order.
fn two(rng: &mut Rng, count: usize) -> (usize, usize) {
    let first = rng.below(count);
    let step = 1 + rng.below(count - 1);
    (first, (first + step) % count)
}

/// How a trait may be laid across the five cards without singling one out: the
/// same on all of them, split three and two, or every one of them different.
/// What is ruled out is four and one, and anything leaving a lone card.
enum Spread {
    Same,
    ThreeTwo,
    AllDifferent,
}

fn scatter(rng: &mut Rng, cards: &mut [Card; CARDS], which: Trait) {
    let span = span_of(which);
    let spread = if span >= CARDS && rng.chance(0.35) {
        Spread::AllDifferent
    } else if rng.chance(0.55) {
        Spread::ThreeTwo
    } else {
        Spread::Same
    };
    let mut slots: Vec<usize> = (0..CARDS).collect();
    rng.shuffle(&mut slots);
    match spread {
        Spread::Same => {
            let only = rng.below(span);
            for card in cards.iter_mut() {
                set(card, which, only);
            }
        }
        Spread::ThreeTwo => {
            let (most, few) = two(rng, span);
            for (rank, slot) in slots.into_iter().enumerate() {
                set(&mut cards[slot], which, if rank < 2 { few } else { most });
            }
        }
        Spread::AllDifferent => {
            let mut values: Vec<usize> = (0..span).collect();
            rng.shuffle(&mut values);
            for (rank, slot) in slots.into_iter().enumerate() {
                set(&mut cards[slot], which, values[rank]);
            }
        }
    }
}

/// A hand where exactly one trait tells exactly one card apart, and no other
/// trait leaves any card on its own.
pub(crate) fn hand(rng: &mut Rng) -> ([Card; CARDS], usize, Trait) {
    let odd_one = rng.below(CARDS);
    let apart = ANSWERABLE[rng.below(ANSWERABLE.len())];

    // Shape and turn never both move. A turn landing on a circle cannot be
    // seen, and a shape that moves takes its turn along with it.
    let turning = match apart {
        Trait::Turn => true,
        Trait::Shape => false,
        _ => rng.chance(0.4),
    };
    let held =
        if turning { TURNERS[rng.below(TURNERS.len())] } else { SHAPES[rng.below(SHAPES.len())] };
    let mut cards = [Card { shape: held, tint: 0, turn: 0, count: 1, pips: 0, tag: 0 }; CARDS];

    for which in TRAITS {
        if which == apart {
            continue;
        }
        match which {
            Trait::Shape if turning => {}
            Trait::Turn if !turning => {}
            _ => scatter(rng, &mut cards, which),
        }
    }

    let (kept, odd) = two(rng, span_of(apart));
    for card in cards.iter_mut() {
        set(card, apart, kept);
    }
    set(&mut cards[odd_one], apart, odd);
    (cards, odd_one, apart)
}

struct Quotients {
    rng: Rng,
    cards: [Card; CARDS],
    odd_one: usize,
    solved: u32,
}

impl Quotients {
    fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let (cards, odd_one, _) = hand(&mut rng);
        Quotients { rng, cards, odd_one, solved: 0 }
    }

    fn corner(slot: usize) -> (f32, f32) {
        let across = CARDS as f32 * WIDE + (CARDS as f32 - 1.0) * GAP;
        (
            frame::WIDTH as f32 * 0.5 - across * 0.5 + slot as f32 * (WIDE + GAP),
            (frame::HEIGHT as f32 - TALL) * 0.5 + 16.0,
        )
    }

    fn under(x: f32, y: f32) -> Option<usize> {
        (0..CARDS).find(|slot| {
            let (cx, cy) = Self::corner(*slot);
            x >= cx && y >= cy && x <= cx + WIDE && y <= cy + TALL
        })
    }
}

/// A card's turn, which is counted in quarters, as an angle.
fn quarters(turn: usize) -> f32 {
    turn as f32 * std::f32::consts::FRAC_PI_2
}

/// The small mark in the corner of a card. Filler, in one flat color so it is
/// never mistaken for the color the answer might turn on.
fn tag(frame: &mut Frame, which: usize, cx: f32, cy: f32) {
    let r = 6.0;
    match which {
        0 => frame.disc(cx, cy, r, palette::MUTED),
        1 => frame.rect(cx - r, cy - 2.0, r * 2.0, 4.0, palette::MUTED),
        2 => frame.cross(cx, cy, r, 3.0, palette::MUTED),
        3 => frame.ring(cx, cy, r, 2.5, palette::MUTED),
        _ => frame.rect(cx - r, cy - r, r * 2.0, r * 2.0, palette::MUTED),
    }
}

fn mark(frame: &mut Frame, card: &Card, cx: f32, cy: f32) {
    let r = 19.0;
    let tint = TINTS[card.tint];
    match card.shape {
        Shape::Round => frame.disc(cx, cy, r, tint),
        Shape::Square => frame.rect(cx - r, cy - r, r * 2.0, r * 2.0, tint),
        Shape::Triangle => {
            let points = [[0.0, -r], [r, r * 0.8], [-r, r * 0.8]];
            frame.spun(&points, (cx, cy), quarters(card.turn), tint);
        }
        Shape::Chevron => {
            let points = [
                [-r, r * 0.3],
                [0.0, -r * 0.7],
                [r, r * 0.3],
                [r * 0.45, r * 0.3],
                [0.0, -r * 0.1],
                [-r * 0.45, r * 0.3],
            ];
            frame.spun(&points, (cx, cy), quarters(card.turn), tint);
        }
    }
}

impl Game for Quotients {
    fn update(&mut self, input: &Input) -> Outcome {
        if !input.button_pressed(Button::Left) {
            return Outcome::Playing;
        }
        let (x, y) = input.cursor();
        let Some(slot) = Quotients::under(x, y) else {
            return Outcome::Playing;
        };
        if slot != self.odd_one {
            return Outcome::Lost;
        }
        self.solved += 1;
        if self.solved == ROUNDS {
            return Outcome::Won;
        }
        let (cards, odd_one, _) = hand(&mut self.rng);
        self.cards = cards;
        self.odd_one = odd_one;
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        let middle = frame::WIDTH as f32 * 0.5;
        frame.text_center(middle, 26.0, "WHICH ONE DOES NOT BELONG", 2, palette::MUTED);
        for slot in 0..CARDS {
            let (x, y) = Quotients::corner(slot);
            frame.bordered(x, y, WIDE, TALL, 3.0, palette::PANEL, palette::EDGE);
            let card = &self.cards[slot];
            let pitch = 44.0;
            let first = y + TALL * 0.5 - (card.count as f32 - 1.0) * pitch * 0.5;
            for step in 0..card.count {
                mark(frame, card, x + WIDE * 0.5, first + step as f32 * pitch);
            }
            tag(frame, card.tag, x + 16.0, y + 16.0);
            let pips = card.pips + 1;
            let width = pips as f32 * 7.0 + (pips as f32 - 1.0) * 4.0;
            for pip in 0..pips {
                let px = x + WIDE * 0.5 - width * 0.5 + pip as f32 * 11.0;
                frame.rect(px, y + TALL - 16.0, 7.0, 7.0, palette::MUTED);
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/quotients.rs"]
mod tests;
