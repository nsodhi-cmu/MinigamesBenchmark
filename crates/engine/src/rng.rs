//! The only source of chance in the crate, so that a seed fixes a whole episode.

#[derive(Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_add(0x9E37_79B9_7F4A_7C15))
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A whole number from 0 up to but not including `bound`.
    pub fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound.max(1) as u64) as usize
    }

    pub fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + self.unit() * (high - low)
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.unit() < probability
    }

    /// `count` of `items`, all different, in the order they were dealt. Asking
    /// for more than there are deals all of them. This is what drawing over and
    /// over until something new comes up would give, without the drawing over.
    pub fn take<T: Copy>(&mut self, items: &[T], count: usize) -> Vec<T> {
        let mut pool = items.to_vec();
        let count = count.min(pool.len());
        for slot in 0..count {
            let pick = slot + self.below(pool.len() - slot);
            pool.swap(slot, pick);
        }
        pool.truncate(count);
        pool
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.below(i + 1));
        }
    }
}

#[cfg(test)]
#[path = "tests/rng.rs"]
mod tests;
