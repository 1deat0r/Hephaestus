//! Deterministic noise source: SplitMix64 + Irwin–Hall Gaussian.
//!
//! Only basic IEEE-754 operations (+,-,*,/) are used, so generated
//! observation streams (and their receipts) are bit-identical on every
//! platform — no libm transcendentals. The 12-uniform sum approximates a
//! standard normal adequately for synthetic fixture noise; it is not a
//! statistical instrument and is never used for scientific inference.

pub struct NoiseSource {
    state: u64,
}

impl NoiseSource {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // SplitMix64 (Steele, Lea, Flood — exact integer arithmetic).
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_uniform(&mut self) -> f64 {
        // 53 high bits scaled into [0, 1).
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    pub fn next_gaussian(&mut self) -> f64 {
        let mut sum = 0.0;
        for _ in 0..12 {
            sum += self.next_uniform();
        }
        sum - 6.0
    }
}
