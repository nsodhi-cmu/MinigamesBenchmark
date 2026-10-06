//! The pixel buffer every game draws into, and the shapes it can draw.

use font8x8::UnicodeFonts;

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 480;

const CELL: u32 = 8;

const PIP: f32 = 6.0;
const PIP_PITCH: f32 = 18.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    const fn packed(self) -> u32 {
        ((self.0 as u32) << 16) | ((self.1 as u32) << 8) | (self.2 as u32)
    }

    /// This color mixed toward `other`, where `t` runs 0 to 1.
    pub fn mix(self, other: Rgb, t: f32) -> Rgb {
        let f = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)) as u8;
        Rgb(f(self.0, other.0), f(self.1, other.1), f(self.2, other.2))
    }
}

pub struct Frame {
    pixels: Vec<u32>,
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    pub fn new() -> Self {
        Frame { pixels: vec![0; (WIDTH * HEIGHT) as usize] }
    }

    /// Packed `0x00RRGGBB`, row major, for a window backend to blit.
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn fill(&mut self, color: Rgb) {
        self.pixels.fill(color.packed());
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Rgb) {
        let (x0, y0) = (x.round() as i32, y.round() as i32);
        let (x1, y1) = ((x + w).round() as i32, (y + h).round() as i32);
        let packed = color.packed();
        for py in y0.max(0)..y1.min(HEIGHT as i32) {
            let row = py as u32 * WIDTH;
            for px in x0.max(0)..x1.min(WIDTH as i32) {
                self.pixels[(row + px as u32) as usize] = packed;
            }
        }
    }

    pub fn outline(&mut self, x: f32, y: f32, w: f32, h: f32, weight: f32, color: Rgb) {
        self.rect(x, y, w, weight, color);
        self.rect(x, y + h - weight, w, weight, color);
        self.rect(x, y, weight, h, color);
        self.rect(x + w - weight, y, weight, h, color);
    }

    pub fn disc(&mut self, cx: f32, cy: f32, r: f32, color: Rgb) {
        self.ring_span(cx, cy, 0.0, r, color);
    }

    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, weight: f32, color: Rgb) {
        self.ring_span(cx, cy, (r - weight).max(0.0), r, color);
    }

    fn ring_span(&mut self, cx: f32, cy: f32, inner: f32, outer: f32, color: Rgb) {
        let (lo, hi) = (inner * inner, outer * outer);
        for py in
            ((cy - outer).floor() as i32).max(0)..((cy + outer).ceil() as i32).min(HEIGHT as i32)
        {
            let dy = py as f32 + 0.5 - cy;
            for px in
                ((cx - outer).floor() as i32).max(0)..((cx + outer).ceil() as i32).min(WIDTH as i32)
            {
                let dx = px as f32 + 0.5 - cx;
                let d = dx * dx + dy * dy;
                if d <= hi && d >= lo {
                    self.pixels[(py as u32 * WIDTH + px as u32) as usize] = color.packed();
                }
            }
        }
    }

    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, weight: f32, color: Rgb) {
        let half = weight * 0.5;
        let (vx, vy) = (x1 - x0, y1 - y0);
        let len2 = vx * vx + vy * vy;
        let bx0 = ((x0.min(x1) - half).floor() as i32).max(0);
        let bx1 = ((x0.max(x1) + half).ceil() as i32).min(WIDTH as i32);
        let by0 = ((y0.min(y1) - half).floor() as i32).max(0);
        let by1 = ((y0.max(y1) + half).ceil() as i32).min(HEIGHT as i32);
        for py in by0..by1 {
            for px in bx0..bx1 {
                let (fx, fy) = (px as f32 + 0.5, py as f32 + 0.5);
                let t = if len2 > 0.0 {
                    (((fx - x0) * vx + (fy - y0) * vy) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let (dx, dy) = (fx - (x0 + vx * t), fy - (y0 + vy * t));
                if dx * dx + dy * dy <= half * half {
                    self.pixels[(py as u32 * WIDTH + px as u32) as usize] = color.packed();
                }
            }
        }
    }

    /// A simple polygon, filled by the even-odd rule.
    pub fn poly(&mut self, points: &[[f32; 2]], color: Rgb) {
        if points.len() < 3 {
            return;
        }
        let top = points.iter().fold(f32::MAX, |a, p| a.min(p[1])).floor() as i32;
        let bottom = points.iter().fold(f32::MIN, |a, p| a.max(p[1])).ceil() as i32;
        let mut crossings = Vec::with_capacity(points.len());
        for py in top.max(0)..bottom.min(HEIGHT as i32) {
            let y = py as f32 + 0.5;
            crossings.clear();
            for i in 0..points.len() {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                if (a[1] > y) != (b[1] > y) {
                    crossings.push(a[0] + (y - a[1]) / (b[1] - a[1]) * (b[0] - a[0]));
                }
            }
            crossings.sort_by(f32::total_cmp);
            for [left, right] in crossings.as_chunks::<2>().0 {
                self.rect(*left, py as f32, right - left, 1.0, color);
            }
        }
    }

    pub fn text(&mut self, x: f32, y: f32, s: &str, scale: u32, color: Rgb) {
        let mut pen = x.round() as i32;
        let top = y.round() as i32;
        for ch in s.chars() {
            if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
                for (row, bits) in glyph.iter().enumerate() {
                    for col in 0..CELL {
                        if bits >> col & 1 == 1 {
                            let (gx, gy) =
                                (pen + (col * scale) as i32, top + (row as u32 * scale) as i32);
                            self.rect(gx as f32, gy as f32, scale as f32, scale as f32, color);
                        }
                    }
                }
            }
            pen += (CELL * scale) as i32;
        }
    }

    /// A filled triangle, its point `reach` above the middle and its base the
    /// same below, `half` either side. A reach the other way turns it over.
    /// Its corners sit on the box around it rather than evenly around a circle,
    /// which is what tells it from a three-cornered `corners`.
    pub fn wedge(&mut self, cx: f32, cy: f32, half: f32, reach: f32, color: Rgb) {
        self.poly(&[[cx, cy - reach], [cx + half, cy + reach], [cx - half, cy + reach]], color);
    }

    /// A shape given as points about the origin, laid about `at` and turned by
    /// `angle`.
    pub fn spun(&mut self, points: &[[f32; 2]], at: (f32, f32), angle: f32, color: Rgb) {
        let (sin, cos) = libm::sincosf(angle);
        let turned: Vec<[f32; 2]> = points
            .iter()
            .map(|[x, y]| [at.0 + x * cos - y * sin, at.1 + x * sin + y * cos])
            .collect();
        self.poly(&turned, color);
    }

    /// Two strokes crossing at a point, each reaching `r` from it, turned by
    /// `spin`.
    pub fn crossed(&mut self, cx: f32, cy: f32, r: f32, weight: f32, spin: f32, color: Rgb) {
        for turn in [spin, spin + std::f32::consts::FRAC_PI_2] {
            let (sin, cos) = libm::sincosf(turn);
            self.line(cx - cos * r, cy - sin * r, cx + cos * r, cy + sin * r, weight, color);
        }
    }

    /// Two diagonal strokes crossing at a point, corner to corner of the square
    /// reaching `r` on either side of it.
    pub fn cross(&mut self, cx: f32, cy: f32, r: f32, weight: f32, color: Rgb) {
        self.line(cx - r, cy - r, cx + r, cy + r, weight, color);
        self.line(cx + r, cy - r, cx - r, cy + r, weight, color);
    }

    /// A filled shape whose corners are spaced evenly around a circle, each
    /// `reaches` from the middle, turned by `spin`. The first corner is at the
    /// top. Reaches that alternate make a star; reaches all the same make a
    /// regular shape, so three is a triangle and four on end is a diamond.
    pub fn corners(&mut self, cx: f32, cy: f32, reaches: &[f32], spin: f32, color: Rgb) {
        let turn = std::f32::consts::TAU / reaches.len() as f32;
        let points: Vec<[f32; 2]> = reaches
            .iter()
            .enumerate()
            .map(|(step, reach)| {
                let angle = spin + step as f32 * turn - std::f32::consts::FRAC_PI_2;
                [cx + libm::cosf(angle) * reach, cy + libm::sinf(angle) * reach]
            })
            .collect();
        self.poly(&points, color);
    }

    /// A filled box inside a border of its own, `weight` deep.
    // Taking the box as a tuple would break with the x, y, w, h of `rect` and `outline`.
    #[allow(clippy::too_many_arguments)]
    pub fn bordered(&mut self, x: f32, y: f32, w: f32, h: f32, weight: f32, fill: Rgb, edge: Rgb) {
        self.rect(x, y, w, h, edge);
        self.rect(x + weight, y + weight, w - weight * 2.0, h - weight * 2.0, fill);
    }

    /// A centered row of pips, the first `done` of them lit: how far along a
    /// player is, or what is left of their chances.
    pub fn pips(&mut self, cx: f32, y: f32, count: u32, done: u32, lit: Rgb) {
        let first = cx - (count.saturating_sub(1)) as f32 * PIP_PITCH * 0.5;
        for slot in 0..count {
            let tint = if slot < done { lit } else { crate::palette::EDGE };
            self.disc(first + slot as f32 * PIP_PITCH, y, PIP, tint);
        }
    }

    pub fn text_center(&mut self, cx: f32, y: f32, s: &str, scale: u32, color: Rgb) {
        self.text(cx - text_width(s, scale) * 0.5, y, s, scale, color);
    }

    pub fn to_png(&self) -> Vec<u8> {
        let mut rgb = Vec::with_capacity(self.pixels.len() * 3);
        for p in &self.pixels {
            rgb.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, *p as u8]);
        }
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, WIDTH, HEIGHT);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("png header");
        writer.write_image_data(&rgb).expect("png body");
        drop(writer);
        out
    }
}

pub fn text_width(s: &str, scale: u32) -> f32 {
    (s.chars().count() as u32 * CELL * scale) as f32
}

pub fn text_height(scale: u32) -> f32 {
    (CELL * scale) as f32
}

#[cfg(test)]
#[path = "tests/frame.rs"]
mod tests;
