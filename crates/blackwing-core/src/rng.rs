//! Deterministic random number generation.
//!
//! Uses a PCG-based PRNG that produces deterministic sequences from a seed.
//! This is critical for replay, networking, and save/load functionality.

use serde::{Deserialize, Serialize};

/// A deterministic random number generator using PCG algorithm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Create a new RNG with the given seed
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Get the current state (useful for saving/restoring)
    pub fn state(&self) -> u64 {
        self.state
    }

    /// Set the state directly (for replay/restore)
    pub fn set_state(&mut self, state: u64) {
        self.state = state;
    }

    /// Fork this RNG into a new independent RNG.
    /// Both the parent and child will produce different sequences after forking.
    pub fn fork(&mut self) -> Rng {
        let seed = self.next_u64();
        // XOR with a constant to ensure parent and child diverge
        Rng::new(seed ^ 0x5851F42D4C957F2D)
    }

    /// Generate the next u64 value
    pub fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    /// Generate a random f64 in [0, 1)
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Generate a random u32 in [0, max)
    pub fn next_u32_range(&mut self, max: u32) -> u32 {
        if max == 0 {
            return 0;
        }
        ((self.next_u64() >> 32) as u32) % max
    }

    /// Generate a random i64 in [min, max)
    pub fn next_i64_range(&mut self, min: i64, max: i64) -> i64 {
        if min >= max {
            return min;
        }
        let range = (max - min) as u64;
        min + (self.next_u64() % range) as i64
    }

    /// Choose a random item from a slice
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        let idx = self.next_u32_range(items.len() as u32) as usize;
        Some(&items[idx])
    }

    /// Choose a random index using weighted selection
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

    /// Shuffle a slice in place
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.next_u32_range((i + 1) as u32) as usize;
            items.swap(i, j);
        }
    }

    /// Roll a check against a probability (0.0 to 1.0)
    pub fn chance(&mut self, probability: f64) -> bool {
        self.next_f64() < probability
    }

    /// Roll dice: returns sum of `count` dice with `sides` faces (1 to sides)
    pub fn roll_dice(&mut self, count: u32, sides: u32) -> u32 {
        if sides == 0 {
            return 0;
        }
        (0..count)
            .map(|_| self.next_u32_range(sides) + 1)
            .sum()
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

    #[test]
    fn fork_produces_different_sequence() {
        let mut rng1 = Rng::new(12345);
        let mut rng2 = rng1.fork();

        // After forking, sequences should differ
        assert_ne!(rng1.next_u64(), rng2.next_u64());
    }

    #[test]
    fn weighted_choice_respects_weights() {
        let mut rng = Rng::new(42);
        let items = ["rare", "common", "common"];
        let weights = [1, 10, 10];

        let mut rare_count = 0;
        let mut common_count = 0;

        for _ in 0..1000 {
            match rng.weighted_choice(&items, &weights) {
                Some(0) => rare_count += 1,
                Some(_) => common_count += 1,
                None => panic!("Should always return Some"),
            }
        }

        // Rare should be much less common
        assert!(rare_count < common_count / 5);
    }

    #[test]
    fn dice_roll_in_range() {
        let mut rng = Rng::new(777);
        for _ in 0..100 {
            let roll = rng.roll_dice(2, 6); // 2d6
            assert!(roll >= 2 && roll <= 12);
        }
    }
}
