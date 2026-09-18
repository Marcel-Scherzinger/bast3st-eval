use serde::{Deserialize, Serialize};

/// This specifies if e. g. a test is allowed to generate new random numbers
/// in addition to (maybe) predefined ones.
///
/// _This is independent of predefined (fixed) random numbers and only influences
/// if the test is stopped if it requests more random numbers then provided or
/// if the request is satisfied._
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(from = "RandomGenerationRepr", into = "RandomGenerationRepr")]
pub enum RandomGeneration {
    /// (default) disable all random number generation capabilities.
    #[default]
    Disabled,
    /// enable generation of new random numbers
    Enabled,
    /// enable generation of new random numbers and use the specified
    /// seed for reproducible results (as long as the implementation doesn't change)
    ///
    /// See [`rand::SeedableRng::seed_from_u64`](https://docs.rs/rand/latest/rand/trait.SeedableRng.html#method.seed_from_u64)
    /// for the function that will likely be used with this value
    EnabledWithSeed(u64),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum RandomGenerationRepr {
    IsEnabled(bool),
    Seed(u64),
}
impl From<RandomGeneration> for RandomGenerationRepr {
    fn from(value: RandomGeneration) -> Self {
        match value {
            RandomGeneration::EnabledWithSeed(s) => Self::Seed(s),
            RandomGeneration::Enabled => Self::IsEnabled(true),
            RandomGeneration::Disabled => Self::IsEnabled(false),
        }
    }
}
impl From<RandomGenerationRepr> for RandomGeneration {
    fn from(value: RandomGenerationRepr) -> Self {
        match value {
            RandomGenerationRepr::Seed(s) => Self::EnabledWithSeed(s),
            RandomGenerationRepr::IsEnabled(true) => Self::Enabled,
            RandomGenerationRepr::IsEnabled(false) => Self::Disabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RandomGeneration;
    #[test]
    fn test_random_generation_serde() {
        let enabled: RandomGeneration = serde_json::from_str("true").unwrap();
        let disabled: RandomGeneration = serde_json::from_str("false").unwrap();
        let seed: RandomGeneration = serde_json::from_str("42").unwrap();
        assert_eq!(RandomGeneration::Enabled, enabled);
        assert_eq!(RandomGeneration::Disabled, disabled);
        assert_eq!(RandomGeneration::EnabledWithSeed(42), seed);
    }
}
