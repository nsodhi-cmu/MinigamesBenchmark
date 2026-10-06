//! Shoot down a row of saucers sweeping closer, from behind shields they wear away.

use crate::frame::{self, Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK, Tag};
use crate::input::{Input, Key};
use crate::palette;
use crate::shape;
use crate::sky;

pub const SPEC: Spec = Spec {
    name: "shuttle",
    title: "Shuttle Shootout",
    brief: "A row of ten saucers sweeps from side to side above you, dropping a step closer \
            each time it reaches the side, and sweeping faster as it thins out. Move along the \
            bottom and fire upward to shoot all ten down. Only one of your shots can be in the \
            air at a time. The saucers fire back. One hit ends the round, and so does the row \
            reaching the dashed line. Three shields stand between you and the saucers, and \
            every shot that hits a shield wears a piece of it away, theirs a bigger \
            piece than yours.",
    controls: &["LEFT and RIGHT move", "SPACE fires"],
    pointer: false,
    scoring: Scoring::Time,
    varies: false,
    tags: &[Tag::Bounded, Tag::Lethal, Tag::Timing],
};

pub fn build(_seed: u64) -> Box<dyn Game> {
    Box::new(Shuttle::new())
}

pub(crate) const SAUCERS: usize = 10;
const SPACING: f32 = 50.0;
pub(crate) const SAUCER_WIDE: f32 = 34.0;
const SAUCER_TALL: f32 = 20.0;
const SIDE_MARGIN: f32 = 14.0;
const DROP: f32 = 14.0;
const SWEEP: f32 = 36.0;
const QUICKENING: f32 = 10.0;
const LANDING: f32 = 356.0;

pub(crate) const SHIP_Y: f32 = 450.0;
pub(crate) const SHIP_PACE: f32 = 220.0;
const SHOT_PACE: f32 = 560.0;

pub(crate) const BOMB_PACE: f32 = 240.0;
const BOMBS: usize = 2;
/// Ticks between the saucers' shots with all ten up, and less for each one down.
const RELOAD: u32 = 66;
const RELOAD_GAIN: u32 = 3;

const SHIELDS: [f32; 3] = [160.0, 320.0, 480.0];
const SHIELD_TOP: f32 = 380.0;
const BRICK: f32 = 6.0;
const BRICKS_ACROSS: usize = 12;
const BRICKS_DOWN: usize = 5;

const HULL: Rgb = Rgb(150, 110, 230);

type Shield = [[bool; BRICKS_ACROSS]; BRICKS_DOWN];

fn whole_shield() -> Shield {
    let mut shield = [[true; BRICKS_ACROSS]; BRICKS_DOWN];
    for row in shield.iter_mut().skip(BRICKS_DOWN - 2) {
        for brick in row.iter_mut().take(8).skip(4) {
            *brick = false;
        }
    }
    shield
}

#[derive(Clone)]
pub(crate) struct Shuttle {
    pub(crate) ship: f32,
    heading: f32,
    pub(crate) up: [bool; SAUCERS],
    /// The left edge of the first saucer's place and the top of the row.
    pub(crate) row: (f32, f32),
    sweeping: f32,
    pub(crate) shot: Option<(f32, f32)>,
    pub(crate) bombs: Vec<(f32, f32)>,
    shields: [Shield; 3],
    reload: u32,
}

impl Shuttle {
    pub(crate) fn new() -> Self {
        let span = SPACING * (SAUCERS - 1) as f32 + SAUCER_WIDE;
        Shuttle {
            ship: frame::WIDTH as f32 * 0.5,
            heading: 0.0,
            up: [true; SAUCERS],
            row: ((frame::WIDTH as f32 - span) * 0.5, 30.0),
            sweeping: 1.0,
            shot: None,
            bombs: Vec::new(),
            shields: [whole_shield(); 3],
            reload: RELOAD,
        }
    }

    fn down(&self) -> u32 {
        self.up.iter().filter(|up| !**up).count() as u32
    }

    pub(crate) fn saucer_middle(&self, which: usize) -> f32 {
        self.row.0 + which as f32 * SPACING + SAUCER_WIDE * 0.5
    }

    /// Takes away the shield brick under a point and the one below it, which
    /// is what a saucer's shot does to a shield, answering whether there was one.
    fn bite(&mut self, at: (f32, f32)) -> bool {
        let struck = self.chip(at);
        if struck {
            self.chip((at.0, at.1 + BRICK));
        }
        struck
    }

    /// Takes away the shield brick under a point, answering whether there was one.
    fn chip(&mut self, at: (f32, f32)) -> bool {
        for (shield, middle) in self.shields.iter_mut().zip(SHIELDS) {
            let column = ((at.0 - (middle - BRICK * BRICKS_ACROSS as f32 * 0.5)) / BRICK).floor();
            let row = ((at.1 - SHIELD_TOP) / BRICK).floor();
            if (0.0..BRICKS_ACROSS as f32).contains(&column)
                && (0.0..BRICKS_DOWN as f32).contains(&row)
                && shield[row as usize][column as usize]
            {
                shield[row as usize][column as usize] = false;
                return true;
            }
        }
        false
    }

    /// The saucer nearest above where the ship will be when a shot from the row
    /// reaches it, going the way it is going now.
    fn shooter(&self) -> Option<usize> {
        let fall = (SHIP_Y - self.row.1 - SAUCER_TALL) / BOMB_PACE;
        let aim = self.ship + self.heading * SHIP_PACE * fall;
        (0..SAUCERS).filter(|which| self.up[*which]).min_by(|a, b| {
            let from = |w: usize| (self.saucer_middle(w) - aim).abs();
            from(*a).total_cmp(&from(*b))
        })
    }
}

impl Game for Shuttle {
    fn update(&mut self, input: &Input) -> Outcome {
        self.heading = input.axis(Key::Left, Key::Right);
        self.ship = (self.ship + self.heading * SHIP_PACE * TICK)
            .clamp(sky::SHIP_HALF, frame::WIDTH as f32 - sky::SHIP_HALF);
        if input.pressed(Key::Space) && self.shot.is_none() {
            self.shot = Some((self.ship, SHIP_Y - sky::SHIP_TALL));
        }

        if let Some((x, was)) = self.shot {
            let y = was - SHOT_PACE * TICK;
            // The stretch the shot covered this tick, not only where it ended:
            // a shot climbs farther in a tick than a saucer is deep, so one it
            // passed clean through would otherwise go unhit.
            let hit = (0..SAUCERS).find(|which| {
                let middle = (self.saucer_middle(*which), self.row.1 + SAUCER_TALL * 0.5);
                self.up[*which]
                    && shape::stroke_hits(
                        (x, was),
                        (x, y),
                        middle,
                        (SAUCER_WIDE * 0.5, SAUCER_TALL * 0.5),
                    )
            });
            self.shot = Some((x, y));
            if let Some(which) = hit {
                self.up[which] = false;
                self.shot = None;
            } else if y < 0.0 || self.chip((x, y)) {
                self.shot = None;
            }
        }
        if self.down() == SAUCERS as u32 {
            return Outcome::Won;
        }

        let first = self.up.iter().position(|up| *up).expect("a saucer is still up");
        let last = self.up.iter().rposition(|up| *up).expect("a saucer is still up");
        let step = self.sweeping * (SWEEP + QUICKENING * self.down() as f32) * TICK;
        let left = self.row.0 + first as f32 * SPACING + step;
        let right = self.row.0 + last as f32 * SPACING + SAUCER_WIDE + step;
        if left < SIDE_MARGIN || right > frame::WIDTH as f32 - SIDE_MARGIN {
            self.row.1 += DROP;
            self.sweeping = -self.sweeping;
        } else {
            self.row.0 += step;
        }
        if self.row.1 + SAUCER_TALL >= LANDING {
            return Outcome::Lost;
        }

        self.reload = self.reload.saturating_sub(1);
        if self.reload == 0 && self.bombs.len() < BOMBS {
            if let Some(which) = self.shooter() {
                self.bombs.push((self.saucer_middle(which), self.row.1 + SAUCER_TALL));
            }
            self.reload = RELOAD - RELOAD_GAIN * self.down();
        }
        let mut kept = Vec::new();
        for (x, y) in std::mem::take(&mut self.bombs) {
            let y = y + BOMB_PACE * TICK;
            if (x - self.ship).abs() <= sky::SHIP_HALF
                && (SHIP_Y - sky::SHIP_TALL..SHIP_Y).contains(&y)
            {
                return Outcome::Lost;
            }
            if y < frame::HEIGHT as f32 && !self.bite((x, y)) {
                kept.push((x, y));
            }
        }
        self.bombs = kept;
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        sky::night(frame);
        let mut dash = 0.0;
        while dash < frame::WIDTH as f32 {
            frame.rect(dash, LANDING, 12.0, 2.0, palette::BAD);
            dash += 24.0;
        }

        for (which, up) in self.up.iter().enumerate() {
            if !up {
                continue;
            }
            let (left, top) = (self.row.0 + which as f32 * SPACING, self.row.1);
            frame.rect(left + 8.0, top, SAUCER_WIDE - 16.0, 8.0, HULL);
            frame.rect(left, top + 7.0, SAUCER_WIDE, 8.0, HULL);
            frame.rect(left + 6.0, top + 9.0, 4.0, 4.0, palette::BACKDROP);
            frame.rect(left + SAUCER_WIDE - 10.0, top + 9.0, 4.0, 4.0, palette::BACKDROP);
            frame.rect(left + 4.0, top + 15.0, 4.0, 5.0, HULL);
            frame.rect(left + SAUCER_WIDE - 8.0, top + 15.0, 4.0, 5.0, HULL);
        }

        for (shield, middle) in self.shields.iter().zip(SHIELDS) {
            let left = middle - BRICK * BRICKS_ACROSS as f32 * 0.5;
            for (row, bricks) in shield.iter().enumerate() {
                for (column, brick) in bricks.iter().enumerate() {
                    if *brick {
                        let (x, y) =
                            (left + column as f32 * BRICK, SHIELD_TOP + row as f32 * BRICK);
                        frame.rect(x, y, BRICK, BRICK, palette::GOOD);
                    }
                }
            }
        }

        sky::ship(frame, self.ship, SHIP_Y);
        if let Some((x, y)) = self.shot {
            sky::shot(frame, x, y);
        }
        for (x, y) in &self.bombs {
            sky::bomb(frame, *x, *y);
        }
    }
}

#[cfg(test)]
#[path = "../tests/games/shuttle.rs"]
mod tests;
