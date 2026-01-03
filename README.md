# rust-rule-miner 🔍⛏️

[![Crates.io](https://img.shields.io/crates/v/rust-rule-miner.svg)](https://crates.io/crates/rust-rule-miner)
[![Documentation](https://docs.rs/rust-rule-miner/badge.svg)](https://docs.rs/rust-rule-miner)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Automatic rule discovery from historical data** using association rule mining, sequential pattern mining, and graph-based pattern matching.

Discover business rules, recommendations, and patterns from your data without manual rule authoring!

---

## 🎯 Features

- **Association Rule Mining** - Discover "If X then Y" patterns (Apriori, FP-Growth algorithms)
- **Sequential Pattern Mining** - Find time-ordered patterns (A → B → C)
- **Graph-Based Patterns** - Model entity relationships and discover complex patterns
- **Quality Metrics** - Confidence, Support, Lift, Conviction scores for each rule
- **GRL Export** - Generate executable rules for [rust-rule-engine](https://github.com/KSD-CO/rust-rule-engine)
- **Excel/CSV Loading** - Stream large datasets from Excel (.xlsx) and CSV files with ultra-low memory using [excelstream](https://github.com/KSD-CO/excelstream)
- **Visualization** - Export graphs to DOT format for Graphviz

---

## 🚀 Quick Start

```rust
use rust_rule_miner::{RuleMiner, Transaction, MiningConfig};
use chrono::Utc;

// 1. Load historical transactions
let transactions = vec![
    Transaction::new("tx1", vec!["Laptop", "Mouse", "Keyboard"], Utc::now()),
    Transaction::new("tx2", vec!["Laptop", "Mouse"], Utc::now()),
    Transaction::new("tx3", vec!["Laptop", "Mouse", "USB-C Hub"], Utc::now()),
    Transaction::new("tx4", vec!["Phone", "Phone Case"], Utc::now()),
];

// 2. Configure mining parameters
let config = MiningConfig {
    min_support: 0.3,      // 30% of transactions
    min_confidence: 0.7,   // 70% confidence
    min_lift: 1.2,         // 20% above random chance
    max_time_gap: None,
    algorithm: MiningAlgorithm::Apriori,
};

// 3. Mine association rules
let mut miner = RuleMiner::new(config);
miner.add_transactions(transactions)?;

let rules = miner.mine_association_rules()?;

// 4. Display discovered rules
for rule in &rules {
    println!("Rule: {:?} => {:?}", rule.antecedent, rule.consequent);
    println!("  Confidence: {:.1}%", rule.metrics.confidence * 100.0);
    println!("  Support: {:.1}%", rule.metrics.support * 100.0);
    println!("  Lift: {:.2}", rule.metrics.lift);
}

// Output:
// Rule: ["Laptop"] => ["Mouse"]
//   Confidence: 100.0%
//   Support: 75.0%
//   Lift: 1.33
```

---

## 📦 Installation

```toml
[dependencies]
rust-rule-miner = "0.1"
```

---

## 📊 Loading Data from Excel/CSV

Stream large datasets with constant memory usage using [excelstream](https://github.com/KSD-CO/excelstream):

```rust
use rust_rule_miner::data_loader::DataLoader;

// Load from CSV file (ultra-fast, ~1.2M rows/sec)
let transactions = DataLoader::from_csv("sales_data.csv")?;

// Load from Excel file (.xlsx)
let transactions = DataLoader::from_excel("sales_data.xlsx", 0)?;  // 0 = first sheet

// Mine rules from loaded data
let mut miner = RuleMiner::new(config);
miner.add_transactions(transactions)?;
let rules = miner.mine_association_rules()?;
```

**Expected file format:**
```csv
transaction_id,items,timestamp
tx001,"Laptop,Mouse,Keyboard",2024-01-01T10:00:00Z
tx002,"Phone,Phone Case",2024-01-02T11:30:00Z
```

**Memory usage:** ~3-35 MB regardless of file size! 🚀

---

## 🔧 Use Cases

### 1. E-commerce Product Recommendations
```rust
// Load historical purchase data from CSV
let transactions = DataLoader::from_csv("purchase_history.csv")?;

// Discover: "Customers who bought X also bought Y"
let mut miner = RuleMiner::new(config);
miner.add_transactions(transactions)?;

let rules = miner.mine_association_rules()?;
// Result: Laptop (85%) → Mouse, Keyboard (75%) → Monitor
```

### 2. Fraud Detection Pattern Discovery
```rust
// Find patterns unique to fraud cases
let fraud_miner = RuleMiner::new(config);
fraud_miner.add_transactions(fraud_cases)?;

let patterns = fraud_miner.mine_association_rules()?;
// Result: IP_mismatch + unusual_time + high_amount → fraud (90%)
```

### 3. Medical Diagnosis Support
```rust
// Discover: "Symptoms A, B, C → Likely Disease X"
let medical_miner = RuleMiner::new(MiningConfig {
    min_confidence: 0.90,  // High confidence for medical
    ..Default::default()
});
```

### 4. Sequential Pattern Mining
```rust
use std::time::Duration;

// Find time-ordered patterns
let config = MiningConfig {
    max_time_gap: Some(Duration::from_secs(7 * 24 * 3600)),  // 7 days
    ..Default::default()
};

let sequential_patterns = miner.find_sequential_patterns()?;
// Result: Laptop → (2 days) → Mouse → (5 days) → Laptop Bag
```

---

## 🎨 Export to rust-rule-engine

Generate executable GRL rules:

```rust
// Generate GRL code
let grl_code = miner.to_grl(&rules);

// Save to file
std::fs::write("mined_rules.grl", grl_code)?;

// Load into rust-rule-engine
use rust_rule_engine::RuleEngine;

let mut engine = RuleEngine::new();
engine.add_rules_from_grl(&grl_code)?;

// Use for recommendations
let mut facts = Facts::new();
facts.set("ShoppingCart.items", vec!["Laptop"]);
engine.execute(&mut facts)?;
```

**Generated GRL (rust-rule-engine v1.15.0+ with `+=` operator):**
```grl
// Auto-generated rules from pattern mining
// Generated: 2026-01-03 14:00:00 UTC

// Rule #1: Laptop → Mouse
// Confidence: 85.7% | Support: 60.0% | Lift: 1.43
rule "Mined_Laptop_Implies_Mouse" salience 85 no-loop {
    when
        ShoppingCart.items contains "Laptop" &&
        !(Recommendation.items contains "Mouse")
    then
        Recommendation.items += "Mouse";  // Array append operator (v1.15.0+)
        LogMessage("Rule fired: confidence 85.7%");
}
```

---

## 📊 Algorithms

### 1. Apriori (Classic)
- **Best for**: Small to medium datasets (<10k transactions)
- **Pros**: Simple, easy to understand, breadth-first search
- **Cons**: Can be slow with many unique items

### 2. FP-Growth (Recommended)
- **Best for**: Large datasets (10k+ transactions)
- **Pros**: Faster than Apriori, no candidate generation
- **Cons**: More complex, uses more memory

### 3. Sequential Pattern Mining
- **Best for**: Time-ordered event sequences
- **Features**: Supports time windows, gap constraints

---

## 🎯 Quality Metrics

Each discovered rule includes:

- **Confidence**: P(B|A) - How often B happens when A happens
- **Support**: P(A ∧ B) - How common the pattern is overall
- **Lift**: Confidence / P(B) - Correlation strength (>1: positive, <1: negative)
- **Conviction**: How much more often A implies B than expected by chance

---

## 📈 Performance

Benchmarks with default config (min_support=0.05, min_confidence=0.6):

| Dataset Size | Algorithm | Time | Memory | Throughput |
|--------------|-----------|------|--------|------------|
| 100 transactions | Apriori | ~10-20ms | ~5 MB | 5-10K tx/s |
| 1,000 transactions | Apriori | ~100-200ms | ~10-15 MB | 5-10K tx/s |
| 10,000 transactions | Apriori | ~1-2s | ~30-50 MB | 5-10K tx/s |
| 100,000 transactions | Apriori | ~10-20s | ~200-500 MB | 5-10K tx/s |

**Notes:**
- Performance varies with min_support threshold (lower = slower)
- Memory usage depends on number of unique items and patterns
- excelstream provides constant ~3-35 MB memory during data loading
- See [docs/PERFORMANCE.md](docs/PERFORMANCE.md) for detailed benchmarks

---

## 🔗 Integration with rust-rule-engine

This crate is designed to work seamlessly with [rust-rule-engine v1.15.0+](https://github.com/KSD-CO/rust-rule-engine):

1. **Mine rules** from historical data (this crate)
2. **Export to GRL** format with `+=` array append operator
3. **Execute rules** with RETE algorithm (rust-rule-engine)
4. **Explain decisions** with backward chaining (rust-rule-engine)

**Requirements:** rust-rule-engine v1.15.0 or higher (for `+=` operator support)

```toml
[dev-dependencies]
rust-rule-engine = "1.15.0"  # Required for += array append in GRL
```

---

## 📚 Examples

See [examples/](examples/) directory:

- `basic_mining.rs` - Simple association rule mining
- `ecommerce_recommendations.rs` - Product recommendation system
- `fraud_detection.rs` - Fraud pattern discovery
- `sequential_patterns.rs` - Time-ordered pattern mining
- `graph_patterns.rs` - Graph-based pattern matching

---

## 🗺️ Roadmap

- [x] Apriori algorithm
- [x] Association rule generation
- [x] Quality metrics (confidence, support, lift)
- [x] GRL export
- [ ] FP-Growth algorithm (Week 3-4)
- [ ] Sequential pattern mining (Week 6)
- [ ] Graph pattern matching (Week 8)
- [ ] Incremental mining
- [ ] Multi-level mining (category hierarchies)
- [ ] Negative pattern mining

---

## 📖 Documentation

- [API Documentation](https://docs.rs/rust-rule-miner)
- [User Guide](docs/USER_GUIDE.md)
- [Algorithm Details](docs/ALGORITHMS.md)
- [Planning Document](docs/planning/GRAPH_BASED_RULE_MINING.md)

---

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 📄 License

MIT License - see [LICENSE](LICENSE) file.

---

## 🔬 Research & References

- **Apriori**: Agrawal & Srikant (VLDB 1994) - "Fast Algorithms for Mining Association Rules"
- **FP-Growth**: Han et al. (SIGMOD 2000) - "Mining Frequent Patterns without Candidate Generation"
- **Sequential Patterns**: Agrawal & Srikant (ICDE 1995) - "Mining Sequential Patterns"

---

## 🌟 Related Projects

- [rust-rule-engine](https://github.com/KSD-CO/rust-rule-engine) - Production rule engine with RETE algorithm
- [mlxtend](https://github.com/rasbt/mlxtend) - Python ML library (inspiration)

---

**Built with ❤️ in Rust 🦀**
