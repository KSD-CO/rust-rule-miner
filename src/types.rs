use serde::{Deserialize, Serialize};
use std::time::Duration;

/// An itemset (set of items)
pub type ItemSet = Vec<String>;

/// Frequent itemset with support value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequentItemset {
    pub items: ItemSet,
    pub support: f64,
}

/// Association rule: A → B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationRule {
    pub antecedent: ItemSet,
    pub consequent: ItemSet,
    pub metrics: PatternMetrics,
}

/// Sequential pattern (ordered itemsets with time constraints)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequentialPattern {
    pub sequence: Vec<ItemSet>,
    pub time_gaps: Vec<Duration>,
    pub support: f64,
}

/// Pattern quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetrics {
    /// Confidence: P(consequent | antecedent)
    /// How often B happens when A happens
    pub confidence: f64,

    /// Support: P(antecedent ∧ consequent)
    /// How common the pattern is overall
    pub support: f64,

    /// Lift: confidence / P(consequent)
    /// Strength of correlation (>1: positive, <1: negative, =1: independent)
    pub lift: f64,

    /// Conviction: P(A) * P(¬B) / P(A ∧ ¬B)
    /// How much more often A implies B than expected by chance
    pub conviction: f64,

    /// Optional: time-based metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_time_gap: Option<Duration>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_variance: Option<Duration>,
}

/// Discovered pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub items: Vec<String>,
    pub metrics: PatternMetrics,
    pub evidence: Vec<String>, // Transaction IDs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    /// Frequent itemset (co-occurrence)
    FrequentItemset,

    /// Sequential pattern (ordered)
    Sequential { time_constraints: Vec<Duration> },

    /// Association rule (A → B)
    AssociationRule {
        antecedent: Vec<String>,
        consequent: Vec<String>,
    },
}

impl AssociationRule {
    /// Calculate quality score for ranking
    pub fn quality_score(&self) -> f64 {
        // Weighted combination of metrics
        self.metrics.confidence * 0.5 + self.metrics.lift * 0.3 + self.metrics.support * 0.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_score() {
        let rule = AssociationRule {
            antecedent: vec!["A".to_string()],
            consequent: vec!["B".to_string()],
            metrics: PatternMetrics {
                confidence: 0.8,
                support: 0.6,
                lift: 1.5,
                conviction: 2.0,
                avg_time_gap: None,
                time_variance: None,
            },
        };

        let score = rule.quality_score();
        assert!(score > 0.0 && score <= 1.0);
    }
}
