use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A simplified RNG manager that provides deterministic seeding
#[derive(Debug, Clone)]
pub struct RngManager {
    master_seed: u64,
}

impl RngManager {
    pub fn new(seed: u64) -> Self {
        Self { master_seed: seed }
    }

    pub fn master_seed(&self) -> u64 {
        self.master_seed
    }

    // Get a new RNG for a component by hashing its name with master seed
    pub fn get_rng(&self, name: &str) -> ChaCha8Rng {
        let mut hasher = DefaultHasher::new();
        self.master_seed.hash(&mut hasher);
        name.hash(&mut hasher);
        ChaCha8Rng::seed_from_u64(hasher.finish())
    }
}

// Simple trait for adding RNG to components
pub trait WithRng {
    fn with_rng(self, rng: ChaCha8Rng) -> Self;
}
