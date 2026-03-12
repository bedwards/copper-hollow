use serde::{Deserialize, Serialize};

/// Top-level composition orchestrator.
/// Holds the RNG seed so that composition is fully deterministic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Composer {
    seed: u64,
}

impl Composer {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use rand::RngExt;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn same_seed_produces_same_output() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        let notes1: Vec<u8> = (0..32).map(|_| rng1.random_range(40..80)).collect();
        let notes2: Vec<u8> = (0..32).map(|_| rng2.random_range(40..80)).collect();

        assert_eq!(notes1, notes2, "same seed must produce identical output");
    }

    #[test]
    fn different_seeds_produce_different_output() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(99);

        let notes1: Vec<u8> = (0..32).map(|_| rng1.random_range(40..80)).collect();
        let notes2: Vec<u8> = (0..32).map(|_| rng2.random_range(40..80)).collect();

        assert_ne!(notes1, notes2, "different seeds should differ");
    }
}
