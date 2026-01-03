use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Mining configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    /// Minimum support threshold (0.0 - 1.0)
    /// Example: 0.1 = pattern must appear in at least 10% of transactions
    pub min_support: f64,

    /// Minimum confidence threshold (0.0 - 1.0)
    /// Example: 0.7 = rule must be correct at least 70% of the time
    pub min_confidence: f64,

    /// Minimum lift threshold
    /// Example: 1.2 = items must co-occur 20% more than random chance
    pub min_lift: f64,

    /// Maximum time gap for sequential patterns
    pub max_time_gap: Option<Duration>,

    /// Mining algorithm to use
    pub algorithm: MiningAlgorithm,
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            min_support: 0.1,    // 10%
            min_confidence: 0.7, // 70%
            min_lift: 1.0,       // No negative correlation
            max_time_gap: None,
            algorithm: MiningAlgorithm::Apriori,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiningAlgorithm {
    /// Apriori algorithm (classic, easy to understand)
    Apriori,

    /// FP-Growth (faster, more memory efficient)
    #[allow(dead_code)]
    FPGrowth,

    /// Eclat (uses vertical data format)
    #[allow(dead_code)]
    Eclat,
}
