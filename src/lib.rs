//! # rust-rule-miner
//!
//! Automatic rule discovery from historical data using association rule mining,
//! sequential pattern mining, and graph-based pattern matching.
//!
//! ## Quick Start
//!
//! ```rust
//! use rust_rule_miner::{RuleMiner, Transaction, MiningConfig, MiningAlgorithm};
//! use chrono::Utc;
//!
//! // Load transactions
//! let transactions = vec![
//!     Transaction::new("tx1", vec!["Laptop".to_string(), "Mouse".to_string()], Utc::now()),
//!     Transaction::new("tx2", vec!["Laptop".to_string(), "Keyboard".to_string()], Utc::now()),
//! ];
//!
//! // Configure mining
//! let config = MiningConfig {
//!     min_support: 0.3,
//!     min_confidence: 0.7,
//!     min_lift: 1.0,
//!     max_time_gap: None,
//!     algorithm: MiningAlgorithm::Apriori,
//! };
//!
//! // Mine rules
//! let mut miner = RuleMiner::new(config);
//! miner.add_transactions(transactions).unwrap();
//! let rules = miner.mine_association_rules().unwrap();
//! ```

pub mod config;
pub mod errors;
pub mod transaction;
pub mod types;

// Mining algorithms
pub mod mining;

// Export formats
pub mod export;

// Graph support
pub mod graph;

// Data loading from Excel/CSV
pub mod data_loader;
pub use data_loader::ColumnMapping;

// Rule engine integration
#[cfg(feature = "engine")]
pub mod engine;

// Re-exports
pub use config::{MiningAlgorithm, MiningConfig};
pub use errors::{MiningError, Result};
pub use mining::RuleMiner;
pub use transaction::Transaction;
pub use types::{
    AssociationRule, FrequentItemset, ItemSet, Pattern, PatternMetrics, PatternType,
    SequentialPattern,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let config = MiningConfig::default();
        let miner = RuleMiner::new(config);
        assert!(miner.transaction_count() == 0);
    }
}
