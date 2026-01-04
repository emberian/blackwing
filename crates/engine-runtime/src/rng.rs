pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn state(&self) -> u64 {
        self.state
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn next_u32_range(&mut self, max: u32) -> u32 {
        ((self.next_u64() >> 32) as u32) % max
    }

    pub fn weighted_choice<T>(&mut self, items: &[T], weights: &[u32]) -> Option<usize> {
        if items.is_empty() || weights.len() != items.len() {
            return None;
        }

        let total: u32 = weights.iter().sum();
        if total == 0 {
            return None;
        }

        let mut roll = self.next_u32_range(total);
        for (i, &weight) in weights.iter().enumerate() {
            if roll < weight {
                return Some(i);
            }
            roll -= weight;
        }

        Some(items.len() - 1)
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_sequence() {
        let mut rng1 = Rng::new(12345);
        let mut rng2 = Rng::new(12345);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn f64_in_range() {
        let mut rng = Rng::new(99999);
        for _ in 0..1000 {
            let val = rng.next_f64();
            assert!((0.0..1.0).contains(&val));
        }
    }
}
