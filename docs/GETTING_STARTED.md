# Getting Started with rust-rule-miner 🔍⛏️

A comprehensive guide to automatic rule discovery from historical data.

---

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Core Concepts](#core-concepts)
4. [Loading Data](#loading-data)
5. [Mining Association Rules](#mining-association-rules)
6. [Exporting Rules](#exporting-rules)
7. [Integration with rust-rule-engine](#integration-with-rust-rule-engine)
8. [Configuration Guide](#configuration-guide)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-rule-miner = "0.1"

# For integration with rule execution
[dev-dependencies]
rust-rule-engine = "1.15.0"  # Required for += operator support
```

---

## Quick Start

### 1. Basic Example: E-commerce Recommendations

```rust
use rust_rule_miner::{RuleMiner, Transaction, MiningConfig, MiningAlgorithm};
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create historical transaction data
    let transactions = vec![
        Transaction::new("tx1", vec!["Laptop", "Mouse", "Keyboard"], Utc::now()),
        Transaction::new("tx2", vec!["Laptop", "Mouse"], Utc::now()),
        Transaction::new("tx3", vec!["Laptop", "Mouse", "USB-C Hub"], Utc::now()),
        Transaction::new("tx4", vec!["Phone", "Phone Case"], Utc::now()),
        Transaction::new("tx5", vec!["Phone", "Phone Case", "Screen Protector"], Utc::now()),
    ];

    // Step 2: Configure mining parameters
    let config = MiningConfig {
        min_support: 0.4,      // 40% - item must appear in 40% of transactions
        min_confidence: 0.75,  // 75% - rule must be correct 75% of the time
        min_lift: 1.2,         // 20% better than random chance
        max_time_gap: None,    // No time constraints
        algorithm: MiningAlgorithm::Apriori,
    };

    // Step 3: Create miner and add transactions
    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;

    // Step 4: Mine association rules
    let rules = miner.mine_association_rules()?;

    // Step 5: Display discovered rules
    println!("Discovered {} rules:\n", rules.len());
    for (i, rule) in rules.iter().enumerate() {
        println!("Rule {}: {:?} => {:?}", i + 1, rule.antecedent, rule.consequent);
        println!("  Confidence: {:.1}%", rule.metrics.confidence * 100.0);
        println!("  Support: {:.1}%", rule.metrics.support * 100.0);
        println!("  Lift: {:.2}", rule.metrics.lift);
        println!("  Interpretation: When customer buys {}, they buy {} {:.1}% of the time\n",
            rule.antecedent.join(" + "),
            rule.consequent.join(" + "),
            rule.metrics.confidence * 100.0
        );
    }

    Ok(())
}
```

**Output:**
```
Discovered 2 rules:

Rule 1: ["Laptop"] => ["Mouse"]
  Confidence: 100.0%
  Support: 60.0%
  Lift: 1.67
  Interpretation: When customer buys Laptop, they buy Mouse 100.0% of the time

Rule 2: ["Phone"] => ["Phone Case"]
  Confidence: 100.0%
  Support: 40.0%
  Lift: 2.50
  Interpretation: When customer buys Phone, they buy Phone Case 100.0% of the time
```

---

## Core Concepts

### 1. Transactions

A **transaction** represents a single purchase event containing:
- **ID**: Unique identifier
- **Items**: List of products/services purchased together
- **Timestamp**: When the transaction occurred

```rust
use rust_rule_miner::Transaction;
use chrono::Utc;

let tx = Transaction::new(
    "tx001",                           // Transaction ID
    vec!["Laptop", "Mouse"],           // Items purchased together
    Utc::now()                         // Timestamp
);
```

### 2. Association Rules

An **association rule** expresses a pattern: "If X, then Y"

- **Antecedent**: The "if" part (what's already in the cart)
- **Consequent**: The "then" part (what to recommend)

Example: `[Laptop] => [Mouse]` means "If customer has Laptop, recommend Mouse"

### 3. Quality Metrics

#### Support
**Definition**: How often the itemset appears in all transactions  
**Formula**: `Support(X) = Count(transactions containing X) / Total transactions`  
**Example**: If "Laptop + Mouse" appears in 60 out of 100 transactions, support = 60%

**Use**: Filter out rare patterns
- Low support (< 10%): Uncommon, might be noise
- High support (> 50%): Very common pattern

#### Confidence
**Definition**: How often the rule is correct  
**Formula**: `Confidence(X => Y) = Support(X ∪ Y) / Support(X)`  
**Example**: If 80 out of 100 "Laptop" buyers also buy "Mouse", confidence = 80%

**Use**: Measure rule reliability
- Low confidence (< 50%): Weak association
- High confidence (> 80%): Strong predictive power

#### Lift
**Definition**: How much better than random chance  
**Formula**: `Lift(X => Y) = Confidence(X => Y) / Support(Y)`  
**Example**: If lift = 2.0, the rule is 2x better than random

**Use**: Find genuine associations
- Lift = 1.0: No association (random)
- Lift > 1.0: Positive association (buy together)
- Lift < 1.0: Negative association (buy separately)

#### Conviction
**Definition**: How often the rule would be incorrect if items were independent  
**Formula**: `Conviction(X => Y) = (1 - Support(Y)) / (1 - Confidence(X => Y))`

**Use**: Measure rule strength
- Conviction = 1.0: No association
- Conviction = ∞: Rule always holds

---

## Loading Data

### From Code (In-Memory)

```rust
use rust_rule_miner::Transaction;
use chrono::Utc;

let transactions = vec![
    Transaction::new("tx1", vec!["A", "B", "C"], Utc::now()),
    Transaction::new("tx2", vec!["A", "B"], Utc::now()),
];
```

### From CSV File

**CSV Format:**
```csv
transaction_id,items,timestamp
tx001,"Laptop,Mouse,Keyboard",2024-01-15T10:30:00Z
tx002,"Laptop,Mouse",2024-01-15T11:00:00Z
tx003,"Phone,Phone Case",2024-01-15T12:00:00Z
```

**Load from CSV:**
```rust
use rust_rule_miner::data_loader::DataLoader;

// Stream data with ultra-low memory (~3-35 MB regardless of file size!)
let transactions = DataLoader::from_csv("sales_data.csv")?;

println!("Loaded {} transactions", transactions.len());
```

### From Excel File

**Excel Format:**
- Column A: Transaction ID
- Column B: Items (comma-separated)
- Column C: Timestamp

**Load from Excel:**
```rust
use rust_rule_miner::data_loader::DataLoader;

// Load from first sheet (index 0)
let transactions = DataLoader::from_excel("sales_data.xlsx", 0)?;

// List all available sheets
let sheets = DataLoader::list_sheets("sales_data.xlsx")?;
for (i, name) in sheets.iter().enumerate() {
    println!("Sheet {}: {}", i, name);
}
```

**Supported Timestamp Formats:**
- ISO 8601: `2024-01-15T10:30:00Z`
- Unix timestamp: `1705316400`
- Common formats: `2024-01-15 10:30:00`, `15/01/2024 10:30:00`

---

## Mining Association Rules

### Basic Mining

```rust
use rust_rule_miner::{RuleMiner, MiningConfig};

let config = MiningConfig {
    min_support: 0.3,
    min_confidence: 0.7,
    ..Default::default()
};

let mut miner = RuleMiner::new(config);
miner.add_transactions(transactions)?;

let rules = miner.mine_association_rules()?;
```

### Filtering Rules

```rust
// Get only high-confidence rules
let high_conf_rules: Vec<_> = rules.iter()
    .filter(|r| r.metrics.confidence > 0.9)
    .collect();

// Get rules with specific antecedent
let laptop_rules: Vec<_> = rules.iter()
    .filter(|r| r.antecedent.contains(&"Laptop".to_string()))
    .collect();

// Sort by lift (best associations first)
let mut sorted_rules = rules.clone();
sorted_rules.sort_by(|a, b| 
    b.metrics.lift.partial_cmp(&a.metrics.lift).unwrap()
);
```

---

## Exporting Rules

### Export to GRL (for rust-rule-engine)

```rust
use rust_rule_miner::export::GrlExporter;
use std::fs;

// Generate GRL code
let grl = GrlExporter::to_grl(&rules);

// Save to file
fs::write("mined_rules.grl", &grl)?;

println!("Exported {} rules to GRL", rules.len());
```

**Generated GRL:**
```grl
// Auto-generated rules from pattern mining
// Generated: 2026-01-03 14:00:00 UTC
// Total rules: 2

// Rule #1: Laptop => Mouse
// Confidence: 100.0% | Support: 60.0% | Lift: 1.67 | Conviction: inf
// Interpretation: When Laptop present, Mouse appears 100.0% of the time
rule "Mined_0_Laptop_Implies_Mouse" salience 100 no-loop {
    when
        ShoppingCart.items contains "Laptop" &&
        !(Recommendation.items contains "Mouse")
    then
        Recommendation.items += "Mouse";
        LogMessage("Rule fired: Mined_0_Laptop_Implies_Mouse (confidence: 100.0%)");
}
```

### Export to JSON

```rust
use serde_json;

let json = serde_json::to_string_pretty(&rules)?;
fs::write("rules.json", json)?;
```

---

## Integration with rust-rule-engine

### Complete Workflow

```rust
use rust_rule_miner::{RuleMiner, MiningConfig, data_loader::DataLoader};
use rust_rule_miner::export::GrlExporter;
use rust_rule_engine::rete::{IncrementalEngine, TypedFacts, FactValue};
use rust_rule_engine::rete::grl_loader::GrlReteLoader;

// STEP 1: Load historical data
let transactions = DataLoader::from_csv("sales_history.csv")?;

// STEP 2: Mine association rules
let config = MiningConfig::default();
let mut miner = RuleMiner::new(config);
miner.add_transactions(transactions)?;
let rules = miner.mine_association_rules()?;

// STEP 3: Export to GRL
let grl = GrlExporter::to_grl(&rules);
std::fs::write("mined_rules.grl", &grl)?;

// STEP 4: Load into RETE engine
let mut engine = IncrementalEngine::new();
GrlReteLoader::load_from_string(&grl, &mut engine)?;

// STEP 5: Use for real-time recommendations
let mut facts = TypedFacts::new();
facts.set("ShoppingCart.items", FactValue::Array(vec![
    FactValue::String("Laptop".to_string())
]));
facts.set("Recommendation.items", FactValue::Array(vec![]));

engine.insert_typed_facts("ShoppingCart", facts.clone());
let fired = engine.fire_all(&mut facts, 10);

if let Some(FactValue::Array(recommendations)) = facts.get("Recommendation.items") {
    println!("Recommendations: {:?}", recommendations);
    // Output: Recommendations: [String("Mouse"), String("Keyboard")]
}
```

---

## Configuration Guide

### MiningConfig Parameters

```rust
pub struct MiningConfig {
    /// Minimum support threshold (0.0 - 1.0)
    pub min_support: f64,
    
    /// Minimum confidence threshold (0.0 - 1.0)
    pub min_confidence: f64,
    
    /// Minimum lift threshold (typically > 1.0)
    pub min_lift: f64,
    
    /// Maximum time gap for sequential patterns (None = no limit)
    pub max_time_gap: Option<Duration>,
    
    /// Algorithm to use (Apriori or FPGrowth)
    pub algorithm: MiningAlgorithm,
}
```

### Recommended Settings

#### E-commerce Product Recommendations
```rust
MiningConfig {
    min_support: 0.05,     // 5% - capture even niche products
    min_confidence: 0.60,  // 60% - balanced accuracy
    min_lift: 1.5,         // 50% better than random
    max_time_gap: None,
    algorithm: MiningAlgorithm::Apriori,
}
```

#### Fraud Detection
```rust
MiningConfig {
    min_support: 0.01,     // 1% - rare fraud patterns
    min_confidence: 0.80,  // 80% - high accuracy needed
    min_lift: 3.0,         // 3x better than random
    max_time_gap: Some(Duration::hours(24)),
    algorithm: MiningAlgorithm::Apriori,
}
```

#### Market Basket Analysis
```rust
MiningConfig {
    min_support: 0.10,     // 10% - common patterns
    min_confidence: 0.50,  // 50% - exploratory
    min_lift: 1.2,         // 20% better than random
    max_time_gap: None,
    algorithm: MiningAlgorithm::FPGrowth,  // Faster for large datasets
}
```

---

## Best Practices

### 1. Data Preparation

**Clean your data:**
```rust
// Remove empty transactions
let cleaned: Vec<_> = transactions.into_iter()
    .filter(|t| !t.items.is_empty())
    .collect();

// Normalize item names
let normalized: Vec<_> = transactions.into_iter()
    .map(|mut t| {
        t.items = t.items.iter()
            .map(|item| item.trim().to_lowercase())
            .collect();
        t
    })
    .collect();
```

### 2. Start Conservative

Begin with **higher thresholds**, then relax:

```rust
// Start here
let config = MiningConfig {
    min_support: 0.3,
    min_confidence: 0.8,
    min_lift: 2.0,
    ..Default::default()
};

// If too few rules, gradually decrease:
// min_support: 0.3 -> 0.2 -> 0.1
// min_confidence: 0.8 -> 0.7 -> 0.6
```

### 3. Monitor Performance

```rust
let stats = miner.get_stats();
println!("Mining Statistics:");
println!("  Transactions processed: {}", stats.transactions_processed);
println!("  Frequent itemsets: {}", stats.frequent_itemsets_count);
println!("  Rules generated: {}", stats.rules_generated);
println!("  Time taken: {:?}", stats.time_elapsed);
```

### 4. Validate Rules

Always validate mined rules with domain experts:

```rust
// Export human-readable report
for rule in &rules {
    if rule.metrics.lift > 3.0 {  // Unusually high
        println!("⚠️  Verify: {:?} => {:?} (lift: {:.2})",
            rule.antecedent, rule.consequent, rule.metrics.lift);
    }
}
```

---

## Troubleshooting

### Problem: No rules found

**Cause**: Thresholds too high

**Solution**:
```rust
// Lower the thresholds
let config = MiningConfig {
    min_support: 0.05,     // Was 0.3
    min_confidence: 0.5,   // Was 0.8
    min_lift: 1.1,         // Was 2.0
    ..Default::default()
};
```

### Problem: Too many rules (thousands)

**Cause**: Thresholds too low

**Solution**:
```rust
// Increase thresholds
let config = MiningConfig {
    min_support: 0.2,      // Was 0.05
    min_confidence: 0.75,  // Was 0.5
    min_lift: 1.5,         // Was 1.1
    ..Default::default()
};

// Or post-filter top N rules
let top_rules: Vec<_> = rules.iter()
    .take(100)  // Top 100 by confidence
    .collect();
```

### Problem: Out of memory

**Cause**: Dataset too large for in-memory processing

**Solution**:
```rust
// Use streaming data loader
use rust_rule_miner::data_loader::DataLoader;

// This uses ~3-35 MB regardless of file size!
let transactions = DataLoader::from_csv("huge_dataset.csv")?;

// Or batch processing
const BATCH_SIZE: usize = 10000;
for chunk in transactions.chunks(BATCH_SIZE) {
    miner.add_transactions(chunk.to_vec())?;
}
```

### Problem: Rules don't make sense

**Cause**: Data quality issues or spurious correlations

**Solution**:
1. Check data quality
2. Increase min_lift threshold
3. Filter by domain knowledge

```rust
// Remove nonsensical rules
let filtered: Vec<_> = rules.into_iter()
    .filter(|r| {
        // Domain-specific filters
        !r.antecedent.is_empty() &&
        !r.consequent.is_empty() &&
        r.metrics.lift > 1.5
    })
    .collect();
```

---

## Next Steps

- 📖 Read [Advanced Topics](ADVANCED.md)
- 🎨 See [Examples](../examples/)
- 🔗 Learn about [Integration Patterns](INTEGRATION.md)
- 📊 Explore [Performance Tuning](PERFORMANCE.md)

---

**Need Help?**
- GitHub Issues: https://github.com/KSD-CO/rust-rule-miner/issues
- Documentation: https://docs.rs/rust-rule-miner
