//! Look at a color, then mix it again from memory with three sliders.

use crate::frame::{Frame, Rgb};
use crate::game::{Game, Outcome, Scoring, Spec, TICK_HZ, Tag};
use crate::input::{Button, Input, Key};
use crate::palette;
use crate::rng::Rng;

pub const SPEC: Spec = Spec {
    name: "hue",
    title: "Hue Hunt",
    brief: "A color is shown for five seconds and then taken away. Three sliders appear, \
            for hue, saturation, and value, and the color they make is shown beside them. \
            Drag them until it matches the color you remember, then press ENTER. Both colors \
            are put side by side for a second, and if yours is too far off the round is over. \
            Three colors to match.",
    controls: &["press on a slider and drag it", "ENTER locks the color in"],
    pointer: true,
    scoring: Scoring::Time,
    varies: true,
    tags: &[Tag::Bounded, Tag::Strict, Tag::Memory, Tag::Drag],
};

pub fn build(seed: u64) -> Box<dyn Game> {
    Box::new(Hue::new(seed))
}

pub(crate) const ROUNDS: usize = 3;
pub(crate) const SHOW: u32 = 5 * TICK_HZ;
pub(crate) const REVEAL: u32 = TICK_HZ;
/// How far apart two colors may be, as a distance in red, green, and blue.
pub(crate) const NEAR: f32 = 64.0;
/// Where every round's sliders start.
const START: [f32; 3] = [0.5, 0.5, 0.5];

pub(crate) const TRACK_X: f32 = 300.0;
pub(crate) const TRACK_W: f32 = 300.0;
pub(crate) const TRACK_Y: [f32; 3] = [150.0, 230.0, 310.0];
const TRACK_H: f32 = 18.0;
const GRAB: f32 = 18.0;
const NAMES: [&str; 3] = ["HUE", "SATURATION", "VALUE"];
const SWATCH: (f32, f32) = (200.0, 180.0);
/// Where the middle of every swatch sits.
const SWATCH_Y: f32 = 480.0 * 0.5 - 10.0;

pub(crate) enum Stage {
    Shown(u32),
    Mixing,
    Revealed(u32),
}

pub(crate) struct Hue {
    rng: Rng,
    pub(crate) round: usize,
    pub(crate) target: [f32; 3],
    pub(crate) mix: [f32; 3],
    grabbed: Option<usize>,
    pub(crate) stage: Stage,
    settled: Option<Outcome>,
}

impl Hue {
    pub(crate) fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let target = deal(&mut rng);
        Hue {
            rng,
            round: 0,
            target,
            mix: START,
            grabbed: None,
            stage: Stage::Shown(SHOW),
            settled: None,
        }
    }

    fn judge(&mut self) -> Outcome {
        if apart(hsv(self.mix), hsv(self.target)) > NEAR {
            return Outcome::Lost;
        }
        self.round += 1;
        if self.round == ROUNDS {
            return Outcome::Won;
        }
        self.target = deal(&mut self.rng);
        self.mix = START;
        self.stage = Stage::Shown(SHOW);
        Outcome::Playing
    }
}

/// A color saturated and bright enough that its hue can be seen, and far enough
/// from where the sliders start that leaving them alone is no answer.
fn deal(rng: &mut Rng) -> [f32; 3] {
    loop {
        let color = [rng.unit(), rng.range(0.4, 1.0), rng.range(0.5, 1.0)];
        if apart(hsv(color), hsv(START)) > NEAR * 2.0 {
            return color;
        }
    }
}

pub(crate) fn hsv([h, s, v]: [f32; 3]) -> Rgb {
    let sector = h.rem_euclid(1.0) * 6.0;
    let f = sector.fract();
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    let (r, g, b) = match sector as u32 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    let byte = |c: f32| (c * 255.0).round() as u8;
    Rgb(byte(r), byte(g), byte(b))
}

pub(crate) fn apart(a: Rgb, b: Rgb) -> f32 {
    let d = |x: u8, y: u8| (x as f32 - y as f32).powi(2);
    (d(a.0, b.0) + d(a.1, b.1) + d(a.2, b.2)).sqrt()
}

fn swatch(frame: &mut Frame, cx: f32, cy: f32, color: Rgb, label: &str) {
    let (w, h) = SWATCH;
    frame.bordered(cx - w * 0.5, cy - h * 0.5, w, h, 3.0, color, palette::EDGE);
    frame.text_center(cx, cy - h * 0.5 - 28.0, label, 2, palette::INK);
}

impl Game for Hue {
    fn update(&mut self, input: &Input) -> Outcome {
        if let Some(settled) = self.settled {
            return settled;
        }
        match self.stage {
            Stage::Shown(left) => {
                self.stage = if left > 1 { Stage::Shown(left - 1) } else { Stage::Mixing };
            }
            Stage::Mixing => {
                let (x, y) = input.cursor();
                if input.button_pressed(Button::Left) {
                    self.grabbed = TRACK_Y.iter().position(|track| (y - track).abs() <= GRAB);
                }
                if let Some(which) = self.grabbed {
                    self.mix[which] = ((x - TRACK_X) / TRACK_W).clamp(0.0, 1.0);
                }
                if input.button_released(Button::Left) {
                    self.grabbed = None;
                }
                if input.pressed(Key::Enter) {
                    self.grabbed = None;
                    self.stage = Stage::Revealed(REVEAL);
                }
            }
            Stage::Revealed(left) => {
                if left > 1 {
                    self.stage = Stage::Revealed(left - 1);
                } else {
                    let outcome = self.judge();
                    if outcome != Outcome::Playing {
                        self.settled = Some(outcome);
                    }
                    return outcome;
                }
            }
        }
        Outcome::Playing
    }

    fn draw(&self, frame: &mut Frame) {
        match self.stage {
            Stage::Shown(left) => {
                swatch(frame, 320.0, SWATCH_Y, hsv(self.target), "REMEMBER THIS");
                let (w, h) = SWATCH;
                let y = SWATCH_Y + h * 0.5 + 16.0;
                frame.rect(320.0 - w * 0.5, y, w, 6.0, palette::EDGE);
                frame.rect(320.0 - w * 0.5, y, w * left as f32 / SHOW as f32, 6.0, palette::INK);
            }
            Stage::Mixing => {
                swatch(frame, 150.0, SWATCH_Y, hsv(self.mix), "YOURS");
                for (which, &y) in TRACK_Y.iter().enumerate() {
                    frame.text(TRACK_X, y - 28.0, NAMES[which], 1, palette::MUTED);
                    for step in 0..TRACK_W as u32 {
                        let mut along = self.mix;
                        along[which] = step as f32 / TRACK_W;
                        let x = TRACK_X + step as f32;
                        frame.rect(x, y - TRACK_H * 0.5, 1.0, TRACK_H, hsv(along));
                    }
                    frame.outline(
                        TRACK_X - 1.0,
                        y - TRACK_H * 0.5 - 1.0,
                        TRACK_W + 2.0,
                        TRACK_H + 2.0,
                        1.0,
                        palette::EDGE,
                    );
                    let knob = TRACK_X + self.mix[which] * TRACK_W;
                    frame.bordered(
                        knob - 4.0,
                        y - 16.0,
                        8.0,
                        32.0,
                        2.0,
                        palette::INK,
                        palette::BACKDROP,
                    );
                }
                frame.text_center(320.0, 400.0, "ENTER LOCKS IT IN", 1, palette::MUTED);
            }
            Stage::Revealed(_) => {
                swatch(frame, 200.0, SWATCH_Y, hsv(self.target), "SHOWN");
                swatch(frame, 440.0, SWATCH_Y, hsv(self.mix), "YOURS");
            }
        }
        frame.pips(320.0, 450.0, ROUNDS as u32, self.round as u32, palette::GOOD);
    }
}

#[cfg(test)]
#[path = "../tests/games/hue.rs"]
mod tests;
