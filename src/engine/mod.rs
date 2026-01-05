//! Rule engine integration module
//!
//! This module provides direct integration with rust-rule-engine, allowing
//! mined rules to be executed in real-time without intermediate steps.
//!
//! # Features
//!
//! - Direct conversion of mined rules to executable rules
//! - Real-time rule execution with Facts
//! - Seamless integration with mining pipeline
//! - Support for both Native and RETE engines

#[cfg(feature = "engine")]
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value};

use crate::errors::{MiningError, Result};
use crate::export::{GrlConfig, GrlExporter};
use crate::types::AssociationRule;

#[cfg(feature = "engine")]
/// Rule engine wrapper that integrates mining results with rust-rule-engine
pub struct MiningRuleEngine {
    engine: RustRuleEngine,
    grl_config: GrlConfig,
}

#[cfg(feature = "engine")]
impl MiningRuleEngine {
    /// Create a new rule engine instance
    pub fn new(kb_name: &str) -> Self {
        let kb = KnowledgeBase::new(kb_name);
        let engine = RustRuleEngine::new(kb);
        Self {
            engine,
            grl_config: GrlConfig::default(),
        }
    }

    /// Create a new rule engine with custom GRL configuration
    pub fn with_config(kb_name: &str, grl_config: GrlConfig) -> Self {
        let kb = KnowledgeBase::new(kb_name);
        let engine = RustRuleEngine::new(kb);
        Self { engine, grl_config }
    }

    /// Get the current GRL configuration
    pub fn grl_config(&self) -> &GrlConfig {
        &self.grl_config
    }

    /// Set the GRL configuration
    pub fn set_grl_config(&mut self, config: GrlConfig) {
        self.grl_config = config;
    }

    /// Load mined association rules into the engine
    pub fn load_rules(&mut self, rules: &[AssociationRule]) -> Result<usize> {
        // Generate GRL code with current configuration
        let grl_code = GrlExporter::to_grl_with_config(rules, &self.grl_config);
        let parsed_rules = GRLParser::parse_rules(&grl_code)
            .map_err(|e| MiningError::ExportFailed(format!("Failed to parse GRL: {}", e)))?;

        let mut loaded_count = 0;
        for rule in parsed_rules {
            self.engine
                .knowledge_base()
                .add_rule(rule)
                .map_err(|e| MiningError::ExportFailed(format!("Failed to add rule: {}", e)))?;
            loaded_count += 1;
        }

        Ok(loaded_count)
    }

    /// Execute rules against provided facts
    pub fn execute(&mut self, facts: &Facts) -> Result<ExecutionResult> {
        let result = self
            .engine
            .execute(facts)
            .map_err(|e| MiningError::ExportFailed(format!("Execution failed: {}", e)))?;

        Ok(ExecutionResult {
            rules_fired: result.rules_fired,
            facts: facts.clone(),
        })
    }

    /// Get reference to the underlying engine
    pub fn engine(&self) -> &RustRuleEngine {
        &self.engine
    }

    /// Get mutable reference to the underlying engine
    pub fn engine_mut(&mut self) -> &mut RustRuleEngine {
        &mut self.engine
    }
}

#[cfg(feature = "engine")]
/// Result of rule execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Number of rules that fired
    pub rules_fired: usize,
    /// Facts after execution (may be modified by rules)
    pub facts: Facts,
}

#[cfg(feature = "engine")]
impl ExecutionResult {
    /// Get a value from the facts
    pub fn get(&self, key: &str) -> Option<Value> {
        self.facts.get(key)
    }

    /// Check if any rules fired
    pub fn has_fired(&self) -> bool {
        self.rules_fired > 0
    }
}

// Helper functions for creating Facts from common data structures

#[cfg(feature = "engine")]
/// Create Facts from items with configurable field names
///
/// # Arguments
/// * `input_items` - List of items to set in input field
/// * `config` - GRL configuration specifying field names
///
/// # Example
/// ```ignore
/// use rust_rule_miner::engine::facts_from_items;
/// use rust_rule_miner::export::GrlConfig;
///
/// let config = GrlConfig::shopping_cart();
/// let facts = facts_from_items(vec!["Laptop".to_string()], &config);
/// ```
pub fn facts_from_items(input_items: Vec<String>, config: &GrlConfig) -> Facts {
    let facts = Facts::new();
    facts.set(
        &config.input_field,
        Value::Array(input_items.into_iter().map(Value::String).collect()),
    );
    facts.set(&config.output_field, Value::Array(vec![]));
    facts
}

#[cfg(feature = "engine")]
/// Create Facts from a shopping cart (uses default config)
/// Convenience function that uses ShoppingCart.items and Recommendation.items
pub fn facts_from_cart(cart_items: Vec<String>) -> Facts {
    facts_from_items(cart_items, &GrlConfig::default())
}

#[cfg(feature = "engine")]
/// Create Facts from transaction data (uses transaction config)
/// Convenience function that uses Transaction.items and Analysis.recommendations
pub fn facts_from_transaction(items: Vec<String>) -> Facts {
    facts_from_items(items, &GrlConfig::transaction())
}

#[cfg(feature = "engine")]
/// Create Facts with additional metadata
///
/// # Arguments
/// * `input_items` - List of items to set in input field
/// * `config` - GRL configuration specifying field names
/// * `metadata` - Additional key-value pairs to set in Facts
///
/// # Example
/// ```ignore
/// use rust_rule_miner::engine::facts_from_items_with_metadata;
/// use rust_rule_miner::export::GrlConfig;
/// use rust_rule_engine::Value;
///
/// let config = GrlConfig::custom("Order.items", "Upsell.products");
/// let metadata = vec![
///     ("Order.total".to_string(), Value::String("99.99".to_string())),
///     ("Customer.tier".to_string(), Value::String("Gold".to_string())),
/// ];
/// let facts = facts_from_items_with_metadata(
///     vec!["Laptop".to_string()],
///     &config,
///     Some(metadata)
/// );
/// ```
pub fn facts_from_items_with_metadata(
    input_items: Vec<String>,
    config: &GrlConfig,
    metadata: Option<Vec<(String, Value)>>,
) -> Facts {
    let facts = facts_from_items(input_items, config);

    if let Some(meta) = metadata {
        for (key, value) in meta {
            facts.set(&key, value);
        }
    }

    facts
}

#[cfg(test)]
#[cfg(feature = "engine")]
mod tests {
    use super::*;
    use crate::{MiningConfig, RuleMiner, Transaction};
    use chrono::Utc;

    #[test]
    fn test_engine_integration() {
        // Mine rules with clear pattern
        let transactions = vec![
            Transaction::new(
                "tx1",
                vec!["Laptop".to_string(), "Mouse".to_string()],
                Utc::now(),
            ),
            Transaction::new(
                "tx2",
                vec!["Laptop".to_string(), "Mouse".to_string()],
                Utc::now(),
            ),
            Transaction::new(
                "tx3",
                vec!["Laptop".to_string(), "Mouse".to_string()],
                Utc::now(),
            ),
            Transaction::new("tx4", vec!["Laptop".to_string()], Utc::now()),
        ];

        let config = MiningConfig {
            min_support: 0.5,
            min_confidence: 0.7,
            ..Default::default()
        };

        let mut miner = RuleMiner::new(config);
        miner.add_transactions(transactions).unwrap();
        let rules = miner.mine_association_rules().unwrap();

        // Should mine at least one rule (Laptop => Mouse)
        assert!(!rules.is_empty(), "No rules were mined");

        // Load into engine
        let mut engine = MiningRuleEngine::new("TestRules");
        let loaded = engine.load_rules(&rules).unwrap();
        assert_eq!(loaded, rules.len(), "Not all rules were loaded");

        // Execute - basic smoke test that execution completes without error
        let facts = facts_from_cart(vec!["Laptop".to_string()]);
        let result = engine.execute(&facts).unwrap();

        // Verify result structure is valid
        assert!(
            result.get("ShoppingCart.items").is_some(),
            "Input facts missing"
        );
        assert!(
            result.get("Recommendation.items").is_some(),
            "Output facts missing"
        );
    }
}
