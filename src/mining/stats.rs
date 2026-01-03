use serde::{Deserialize, Serialize};

/// Mining statistics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MiningStats {
    pub frequent_itemsets_count: usize,
    pub rules_generated: usize,
    pub transactions_processed: usize,
}

impl MiningStats {
    pub fn new() -> Self {
        Self::default()
    }
}
