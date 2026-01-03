pub mod apriori;
pub mod stats;

use crate::config::MiningConfig;
use crate::errors::{MiningError, Result};
use crate::transaction::Transaction;
use crate::types::{AssociationRule, FrequentItemset, ItemSet, PatternMetrics};

/// Main rule mining engine
pub struct RuleMiner {
    config: MiningConfig,
    transactions: Vec<Transaction>,
    stats: stats::MiningStats,
}

impl RuleMiner {
    /// Create new rule miner with config
    pub fn new(config: MiningConfig) -> Self {
        Self {
            config,
            transactions: Vec::new(),
            stats: stats::MiningStats::default(),
        }
    }

    /// Add transactions to mine
    pub fn add_transactions(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        if transactions.is_empty() {
            return Err(MiningError::InsufficientData(
                "No transactions provided".to_string(),
            ));
        }
        self.transactions.extend(transactions);
        Ok(())
    }

    /// Add transactions from an iterator (streaming support)
    ///
    /// This method allows adding transactions one-by-one from a stream,
    /// maintaining constant memory usage for the loading phase.
    ///
    /// # Example
    /// ```no_run
    /// use rust_rule_miner::{RuleMiner, MiningConfig, data_loader::DataLoader, Transaction};
    /// use chrono::Utc;
    ///
    /// let mut miner = RuleMiner::new(MiningConfig::default());
    ///
    /// // Add transactions one by one
    /// let transaction = Transaction::new("tx1".to_string(), vec!["A".to_string()], Utc::now());
    /// miner.add_transaction(transaction)?;
    ///
    /// let rules = miner.mine_association_rules()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        self.transactions.push(transaction);
        Ok(())
    }

    /// Add transactions from an iterator (batch streaming)
    ///
    /// More efficient than add_transaction() when you have an iterator.
    ///
    /// # Example
    /// ```no_run
    /// use rust_rule_miner::{RuleMiner, MiningConfig, data_loader::DataLoader};
    ///
    /// let mut miner = RuleMiner::new(MiningConfig::default());
    ///
    /// // Load from CSV and add to miner
    /// let transactions = DataLoader::from_csv("file.csv")?;
    /// miner.add_transactions_from_iter(transactions.into_iter().map(Ok))?;
    ///
    /// let rules = miner.mine_association_rules()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_transactions_from_iter<I>(&mut self, iter: I) -> Result<()>
    where
        I: Iterator<Item = Result<Transaction>>,
    {
        let mut count = 0;
        for transaction_result in iter {
            let transaction = transaction_result?;
            self.transactions.push(transaction);
            count += 1;
        }

        if count == 0 {
            return Err(MiningError::InsufficientData(
                "No transactions provided from iterator".to_string(),
            ));
        }

        Ok(())
    }

    /// Get transaction count
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Mine association rules using configured algorithm
    pub fn mine_association_rules(&mut self) -> Result<Vec<AssociationRule>> {
        if self.transactions.is_empty() {
            return Err(MiningError::InsufficientData(
                "No transactions to mine".to_string(),
            ));
        }

        // Step 1: Find frequent itemsets
        let frequent_itemsets = match self.config.algorithm {
            crate::config::MiningAlgorithm::Apriori => {
                apriori::find_frequent_itemsets(&self.transactions, self.config.min_support)?
            }
            _ => {
                return Err(MiningError::MiningFailed(
                    "Algorithm not yet implemented".to_string(),
                ))
            }
        };

        self.stats.frequent_itemsets_count = frequent_itemsets.len();

        // Step 2: Generate association rules
        let mut rules = self.generate_association_rules(&frequent_itemsets)?;

        // Step 3: Filter bidirectional rules to prevent infinite loops
        rules = self.filter_bidirectional_rules(rules);

        self.stats.rules_generated = rules.len();

        Ok(rules)
    }

    /// Filter out bidirectional rules that could cause infinite loops
    /// For rules like A=>B and B=>A, keep only the one with higher confidence
    fn filter_bidirectional_rules(&self, rules: Vec<AssociationRule>) -> Vec<AssociationRule> {
        let mut filtered = Vec::new();
        let mut seen_pairs = std::collections::HashSet::new();

        // Already sorted by quality score from generate_association_rules

        for rule in rules {
            // Create canonical pair representation (sorted to be order-independent)
            let mut pair = vec![rule.antecedent.clone(), rule.consequent.clone()];
            pair.sort();
            let pair_key = format!("{:?}", pair);

            if !seen_pairs.contains(&pair_key) {
                seen_pairs.insert(pair_key);
                filtered.push(rule);
            }
        }

        filtered
    }

    /// Generate association rules from frequent itemsets
    fn generate_association_rules(
        &self,
        frequent_itemsets: &[FrequentItemset],
    ) -> Result<Vec<AssociationRule>> {
        let mut rules = Vec::new();

        for itemset in frequent_itemsets {
            if itemset.items.len() < 2 {
                continue; // Need at least 2 items for a rule
            }

            // Generate all possible splits: A → B where A ∪ B = itemset
            for antecedent in self.generate_non_empty_subsets(&itemset.items) {
                let consequent: ItemSet = itemset
                    .items
                    .iter()
                    .filter(|item| !antecedent.contains(item))
                    .cloned()
                    .collect();

                if consequent.is_empty() {
                    continue;
                }

                // Calculate metrics
                let metrics = self.calculate_metrics(&antecedent, &consequent, itemset.support);

                // Filter by thresholds
                if metrics.confidence >= self.config.min_confidence
                    && metrics.lift >= self.config.min_lift
                {
                    rules.push(AssociationRule {
                        antecedent: antecedent.clone(),
                        consequent: consequent.clone(),
                        metrics,
                    });
                }
            }
        }

        // Sort by quality score
        rules.sort_by(|a, b| {
            b.quality_score()
                .partial_cmp(&a.quality_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(rules)
    }

    /// Generate all non-empty subsets of an itemset
    fn generate_non_empty_subsets(&self, items: &[String]) -> Vec<ItemSet> {
        let mut subsets = Vec::new();
        let n = items.len();

        // Generate all possible combinations (2^n - 1, excluding empty set and full set)
        for i in 1..(1 << n) - 1 {
            let mut subset = Vec::new();
            for (j, item) in items.iter().enumerate() {
                if (i & (1 << j)) != 0 {
                    subset.push(item.clone());
                }
            }
            subsets.push(subset);
        }

        subsets
    }

    /// Calculate metrics for a rule
    fn calculate_metrics(
        &self,
        antecedent: &ItemSet,
        consequent: &ItemSet,
        both_support: f64,
    ) -> PatternMetrics {
        let total = self.transactions.len() as f64;

        // Count occurrences
        let antecedent_count = self
            .transactions
            .iter()
            .filter(|tx| tx.contains_all(antecedent))
            .count() as f64;

        let consequent_count = self
            .transactions
            .iter()
            .filter(|tx| tx.contains_all(consequent))
            .count() as f64;

        let both_count = self
            .transactions
            .iter()
            .filter(|tx| tx.contains_all(antecedent) && tx.contains_all(consequent))
            .count() as f64;

        // Calculate metrics
        let confidence = if antecedent_count > 0.0 {
            both_count / antecedent_count
        } else {
            0.0
        };

        let support = both_support;

        let p_consequent = consequent_count / total;
        let lift = if p_consequent > 0.0 {
            confidence / p_consequent
        } else {
            0.0
        };

        let conviction = if confidence < 1.0 && p_consequent < 1.0 {
            (1.0 - p_consequent) / (1.0 - confidence)
        } else {
            f64::INFINITY
        };

        PatternMetrics {
            confidence,
            support,
            lift,
            conviction,
            avg_time_gap: None,
            time_variance: None,
        }
    }

    /// Get mining statistics
    pub fn stats(&self) -> &stats::MiningStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_basic_mining() {
        let transactions = vec![
            Transaction::new("tx1", vec!["A".to_string(), "B".to_string()], Utc::now()),
            Transaction::new("tx2", vec!["A".to_string(), "B".to_string()], Utc::now()),
            Transaction::new("tx3", vec!["A".to_string(), "C".to_string()], Utc::now()),
        ];

        let config = MiningConfig {
            min_support: 0.5,
            min_confidence: 0.6,
            min_lift: 1.0,
            ..Default::default()
        };

        let mut miner = RuleMiner::new(config);
        miner.add_transactions(transactions).unwrap();

        let rules = miner.mine_association_rules().unwrap();
        assert!(!rules.is_empty());
    }
}
