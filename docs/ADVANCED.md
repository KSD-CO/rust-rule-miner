# Advanced Topics 🚀

Advanced techniques and patterns for rust-rule-miner.

---

## Table of Contents

1. [Sequential Pattern Mining](#sequential-pattern-mining)
2. [Graph-Based Pattern Matching](#graph-based-pattern-matching)
3. [Custom Metrics](#custom-metrics)
4. [Rule Filtering Strategies](#rule-filtering-strategies)
5. [Performance Optimization](#performance-optimization)
6. [Bidirectional Rule Prevention](#bidirectional-rule-prevention)
7. [Multi-Level Mining](#multi-level-mining)
8. [Incremental Mining](#incremental-mining)

---

## Sequential Pattern Mining

Mine time-ordered patterns where sequence matters.

### Basic Sequential Mining

```rust
use rust_rule_miner::{RuleMiner, Transaction, MiningConfig};
use chrono::{Utc, Duration};

// Create time-ordered transactions
let mut transactions = vec![];

let base_time = Utc::now();
transactions.push(Transaction::new(
    "user1_session1",
    vec!["View Product Page"],
    base_time
));
transactions.push(Transaction::new(
    "user1_session1",
    vec!["Add to Cart"],
    base_time + Duration::minutes(5)
));
transactions.push(Transaction::new(
    "user1_session1",
    vec!["Checkout"],
    base_time + Duration::minutes(10)
));

// Configure with max time gap
let config = MiningConfig {
    min_support: 0.2,
    min_confidence: 0.7,
    max_time_gap: Some(Duration::hours(24)),  // Events within 24h
    ..Default::default()
};

let mut miner = RuleMiner::new(config);
miner.add_transactions(transactions)?;

// Mine sequential patterns
let patterns = miner.mine_sequential_patterns()?;

for pattern in patterns {
    println!("Sequential Pattern: {:?}", pattern.sequence);
    println!("  Avg time gap: {:?}", pattern.metrics.avg_time_gap);
    println!("  Support: {:.1}%", pattern.metrics.support * 100.0);
}
```

### Use Cases for Sequential Mining

**1. Customer Journey Analysis**
```rust
// Discover: Login → Browse → Add to Cart → Purchase
// Time gap: Average 2 hours between steps
```

**2. Fraud Detection**
```rust
// Pattern: Multiple failed logins → Password reset → Large transaction
// If detected within 1 hour → Flag as suspicious
```

**3. User Behavior Flows**
```rust
// Pattern: Watch Tutorial → Try Feature → Upgrade Plan
// Optimize: Reduce time between steps
```

---

## Graph-Based Pattern Matching

Model complex entity relationships and discover patterns in graphs.

### Building Entity Graphs

```rust
use rust_rule_miner::graph::{EntityGraph, EntityType, RelationType};

let mut graph = EntityGraph::new();

// Add entities
let customer1 = graph.add_entity("C001", EntityType::Customer);
let product1 = graph.add_entity("P001", EntityType::Product);
let product2 = graph.add_entity("P002", EntityType::Product);

// Add relationships
graph.add_relationship(customer1, product1, RelationType::Purchased);
graph.add_relationship(customer1, product2, RelationType::Purchased);
graph.add_relationship(product1, product2, RelationType::FrequentlyBoughtTogether);

// Mine graph patterns
let patterns = graph.mine_patterns(&MiningConfig {
    min_support: 0.1,
    min_confidence: 0.6,
    ..Default::default()
})?;

for pattern in patterns {
    println!("Graph Pattern: {:?}", pattern.nodes);
    println!("  Confidence: {:.1}%", pattern.metrics.confidence * 100.0);
}
```

### Advanced Graph Queries

```rust
// Find all customers who bought product A and product B
let co_buyers = graph.find_pattern(vec![
    (EntityType::Customer, RelationType::Purchased, EntityType::Product("A")),
    (EntityType::Customer, RelationType::Purchased, EntityType::Product("B")),
])?;

// Export graph for visualization
use std::fs;
let dot = graph.to_dot();
fs::write("graph.dot", dot)?;

// Convert to image: dot -Tpng graph.dot -o graph.png
```

---

## Custom Metrics

Define your own quality metrics beyond the standard ones.

### Implementing Custom Metrics

```rust
use rust_rule_miner::types::{AssociationRule, PatternMetrics};

// Custom metric: Revenue impact
fn calculate_revenue_impact(
    rule: &AssociationRule,
    item_prices: &HashMap<String, f64>
) -> f64 {
    let consequent_value: f64 = rule.consequent.iter()
        .filter_map(|item| item_prices.get(item))
        .sum();
    
    // Weighted by confidence and support
    consequent_value * rule.metrics.confidence * rule.metrics.support
}

// Filter rules by custom metric
let high_value_rules: Vec<_> = rules.iter()
    .filter(|r| calculate_revenue_impact(r, &prices) > 100.0)
    .collect();

// Sort by revenue impact
let mut sorted = rules.clone();
sorted.sort_by(|a, b| {
    let impact_a = calculate_revenue_impact(a, &prices);
    let impact_b = calculate_revenue_impact(b, &prices);
    impact_b.partial_cmp(&impact_a).unwrap()
});
```

### Popular Custom Metrics

**1. Cosine Similarity**
```rust
fn cosine_similarity(rule: &AssociationRule, all_items: &HashSet<String>) -> f64 {
    let antecedent_set: HashSet<_> = rule.antecedent.iter().collect();
    let consequent_set: HashSet<_> = rule.consequent.iter().collect();
    
    let intersection = antecedent_set.intersection(&consequent_set).count();
    let magnitude_a = (antecedent_set.len() as f64).sqrt();
    let magnitude_b = (consequent_set.len() as f64).sqrt();
    
    intersection as f64 / (magnitude_a * magnitude_b)
}
```

**2. Jaccard Index**
```rust
fn jaccard_index(rule: &AssociationRule) -> f64 {
    let a: HashSet<_> = rule.antecedent.iter().collect();
    let b: HashSet<_> = rule.consequent.iter().collect();
    
    let intersection = a.intersection(&b).count();
    let union = a.union(&b).count();
    
    intersection as f64 / union as f64
}
```

**3. Kulczynski Measure**
```rust
fn kulczynski(rule: &AssociationRule, support_map: &HashMap<String, f64>) -> f64 {
    let p_a_given_b = rule.metrics.confidence;
    let p_b_given_a = rule.metrics.support / support_map.get(&rule.antecedent[0]).unwrap_or(&1.0);
    
    (p_a_given_b + p_b_given_a) / 2.0
}
```

---

## Rule Filtering Strategies

### Bidirectional Rule Prevention

Automatically prevents infinite loops from A→B and B→A rules:

```rust
// This is done automatically in v0.1.0+
let rules = miner.mine_association_rules()?;

// Internally filters bidirectional rules
// Only keeps the rule with higher confidence
// Example: If both exist, keeps only:
//   Laptop → Mouse (confidence: 0.95)
// And removes:
//   Mouse → Laptop (confidence: 0.60)
```

### Redundancy Elimination

Remove rules that are subsumed by more general rules:

```rust
fn remove_redundant_rules(rules: Vec<AssociationRule>) -> Vec<AssociationRule> {
    let mut filtered = Vec::new();
    
    for rule in rules {
        let mut is_redundant = false;
        
        for other in &filtered {
            // Check if rule is a specialization of other
            if is_subset(&other.antecedent, &rule.antecedent) &&
               other.consequent == rule.consequent &&
               other.metrics.confidence >= rule.metrics.confidence {
                is_redundant = true;
                break;
            }
        }
        
        if !is_redundant {
            filtered.push(rule);
        }
    }
    
    filtered
}

fn is_subset(subset: &[String], superset: &[String]) -> bool {
    subset.iter().all(|item| superset.contains(item))
}
```

### Statistical Significance Testing

Filter rules by statistical significance:

```rust
use statrs::distribution::{ChiSquared, ContinuousCDF};

fn chi_square_test(rule: &AssociationRule, total_transactions: usize) -> f64 {
    let n = total_transactions as f64;
    let n_ab = (rule.metrics.support * n) as f64;
    let n_a = n_ab / rule.metrics.confidence;
    let n_b = n_ab / (rule.metrics.support / n_a * n);
    
    let expected = (n_a * n_b) / n;
    let chi_square = ((n_ab - expected).powi(2)) / expected;
    
    let dist = ChiSquared::new(1.0).unwrap();
    1.0 - dist.cdf(chi_square)  // p-value
}

// Filter by p-value < 0.05 (95% confidence)
let significant_rules: Vec<_> = rules.into_iter()
    .filter(|r| chi_square_test(r, total_tx) < 0.05)
    .collect();
```

---

## Performance Optimization

### 1. Algorithm Selection

```rust
use rust_rule_miner::MiningAlgorithm;

// Small datasets (< 10K transactions)
let config = MiningConfig {
    algorithm: MiningAlgorithm::Apriori,  // Simple, easier to debug
    ..Default::default()
};

// Large datasets (> 100K transactions)
let config = MiningConfig {
    algorithm: MiningAlgorithm::FPGrowth,  // 10-100x faster
    ..Default::default()
};
```

### 2. Batch Processing

Process large datasets in batches:

```rust
use rust_rule_miner::data_loader::DataLoader;

const BATCH_SIZE: usize = 50_000;

let all_transactions = DataLoader::from_csv("huge_file.csv")?;

for (i, chunk) in all_transactions.chunks(BATCH_SIZE).enumerate() {
    println!("Processing batch {} of {}", i + 1, 
        (all_transactions.len() + BATCH_SIZE - 1) / BATCH_SIZE);
    
    let mut miner = RuleMiner::new(config.clone());
    miner.add_transactions(chunk.to_vec())?;
    
    let batch_rules = miner.mine_association_rules()?;
    // Merge with global rules...
}
```

### 3. Parallel Processing

```rust
use rayon::prelude::*;

// Process multiple datasets in parallel
let datasets = vec!["data1.csv", "data2.csv", "data3.csv"];

let all_rules: Vec<_> = datasets.par_iter()
    .filter_map(|file| {
        let transactions = DataLoader::from_csv(file).ok()?;
        let mut miner = RuleMiner::new(config.clone());
        miner.add_transactions(transactions).ok()?;
        miner.mine_association_rules().ok()
    })
    .flatten()
    .collect();
```

### 4. Memory Optimization

```rust
// Use streaming for large files (constant memory usage)
let transactions = DataLoader::from_csv("large_file.csv")?;
// Uses only ~3-35 MB regardless of file size!

// Clear intermediate data structures
miner.clear_cache()?;  // Free up memory after mining

// Limit rule count
let config = MiningConfig {
    min_support: 0.1,  // Higher = fewer rules = less memory
    ..Default::default()
};
```

---

## Multi-Level Mining

Mine rules at different granularity levels using category hierarchies.

### Defining Category Hierarchies

```rust
use std::collections::HashMap;

// Define product categories
let mut hierarchy = HashMap::new();
hierarchy.insert("Laptop", vec!["Electronics", "Computer"]);
hierarchy.insert("Mouse", vec!["Electronics", "Computer", "Accessories"]);
hierarchy.insert("Phone", vec!["Electronics", "Mobile"]);

// Expand transactions with categories
fn expand_with_categories(
    tx: &Transaction,
    hierarchy: &HashMap<&str, Vec<&str>>
) -> Transaction {
    let mut expanded_items = tx.items.clone();
    
    for item in &tx.items {
        if let Some(categories) = hierarchy.get(item.as_str()) {
            for category in categories {
                expanded_items.push(category.to_string());
            }
        }
    }
    
    Transaction::new(tx.id.clone(), expanded_items, tx.timestamp)
}

// Mine at multiple levels
let expanded: Vec<_> = transactions.iter()
    .map(|tx| expand_with_categories(tx, &hierarchy))
    .collect();

let mut miner = RuleMiner::new(config);
miner.add_transactions(expanded)?;

let multi_level_rules = miner.mine_association_rules()?;

// Now you get rules like:
// Electronics => Accessories (general)
// Computer => Mouse (specific)
```

---

## Incremental Mining

Update rules as new data arrives without re-mining everything.

### Basic Incremental Mining

```rust
// Initial mining
let mut miner = RuleMiner::new(config);
miner.add_transactions(initial_transactions)?;
let initial_rules = miner.mine_association_rules()?;

// New data arrives
let new_transactions = vec![
    Transaction::new("tx100", vec!["Laptop", "Mouse"], Utc::now()),
    Transaction::new("tx101", vec!["Phone", "Charger"], Utc::now()),
];

// Incremental update
miner.add_transactions(new_transactions)?;
let updated_rules = miner.mine_association_rules()?;

// Compare changes
let new_rules: Vec<_> = updated_rules.iter()
    .filter(|r| !initial_rules.contains(r))
    .collect();

println!("Discovered {} new rules from incremental data", new_rules.len());
```

### Efficient Incremental Updates

```rust
// Track only changed itemsets
let prev_frequent = miner.get_frequent_itemsets();

// Add new data
miner.add_transactions(new_batch)?;

// Mine only affected itemsets
let new_frequent = miner.get_frequent_itemsets();
let changed: Vec<_> = new_frequent.iter()
    .filter(|item| !prev_frequent.contains(item))
    .collect();

// Generate rules only from changed itemsets
let incremental_rules = miner.generate_rules_from_itemsets(&changed)?;
```

---

## Best Practices for Advanced Mining

### 1. Domain Knowledge Integration

```rust
// Encode domain constraints
fn is_valid_rule(rule: &AssociationRule, domain: &DomainKnowledge) -> bool {
    // Electronics can recommend accessories
    if domain.is_category(&rule.antecedent, "Electronics") &&
       domain.is_category(&rule.consequent, "Accessories") {
        return true;
    }
    
    // Food cannot recommend electronics
    if domain.is_category(&rule.antecedent, "Food") &&
       domain.is_category(&rule.consequent, "Electronics") {
        return false;
    }
    
    true
}

let domain_filtered: Vec<_> = rules.into_iter()
    .filter(|r| is_valid_rule(r, &domain_knowledge))
    .collect();
```

### 2. A/B Testing Integration

```rust
// Mine rules from control and test groups
let control_rules = mine_rules(&control_transactions)?;
let test_rules = mine_rules(&test_transactions)?;

// Compare performance
fn compare_rule_sets(control: &[AssociationRule], test: &[AssociationRule]) {
    let avg_conf_control: f64 = control.iter()
        .map(|r| r.metrics.confidence)
        .sum::<f64>() / control.len() as f64;
    
    let avg_conf_test: f64 = test.iter()
        .map(|r| r.metrics.confidence)
        .sum::<f64>() / test.len() as f64;
    
    println!("Control avg confidence: {:.1}%", avg_conf_control * 100.0);
    println!("Test avg confidence: {:.1}%", avg_conf_test * 100.0);
    println!("Improvement: {:.1}%", 
        ((avg_conf_test - avg_conf_control) / avg_conf_control) * 100.0);
}
```

### 3. Continuous Monitoring

```rust
use std::time::Instant;

// Monitor mining performance over time
struct MiningMonitor {
    start_time: Instant,
    rules_per_second: f64,
    memory_usage_mb: f64,
}

impl MiningMonitor {
    fn track_mining(&mut self, miner: &RuleMiner) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let stats = miner.get_stats();
        
        self.rules_per_second = stats.rules_generated as f64 / elapsed;
        self.memory_usage_mb = stats.memory_usage_bytes as f64 / 1_048_576.0;
        
        println!("Mining Performance:");
        println!("  Rules/sec: {:.2}", self.rules_per_second);
        println!("  Memory: {:.2} MB", self.memory_usage_mb);
    }
}
```

---

## Next Steps

- 📖 Back to [Getting Started](GETTING_STARTED.md)
- 🔗 Learn about [Integration Patterns](INTEGRATION.md)
- 📊 Explore [Performance Tuning](PERFORMANCE.md)

---

**Questions?**
- GitHub: https://github.com/KSD-CO/rust-rule-miner
- Docs: https://docs.rs/rust-rule-miner
