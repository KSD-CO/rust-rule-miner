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
- **🆕 Engine Integration** - Direct execution with [rust-rule-engine](https://github.com/KSD-CO/rust-rule-engine) (optional `engine` feature)
- **🆕 PostgreSQL Streaming** - Stream and mine data directly from PostgreSQL (optional `postgres` feature)
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
rust-rule-miner = "0.2"

# Optional features
rust-rule-miner = { version = "0.2", features = ["engine"] }      # Rule execution
rust-rule-miner = { version = "0.2", features = ["postgres"] }    # PostgreSQL streaming
rust-rule-miner = { version = "0.2", features = ["cloud"] }       # Cloud storage (S3, HTTP)
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

**Required file format (columns must be in this order):**
```csv
transaction_id,items,timestamp
tx001,"Laptop,Mouse,Keyboard",2024-01-01T10:00:00Z
tx002,"Phone,Phone Case",2024-01-02T11:30:00Z
```

- **Column 0**: Transaction/Group ID
- **Column 1**: Items (comma-separated values to mine)
- **Column 2**: Timestamp (ISO 8601, Unix timestamp, or datetime string)

**Memory usage:** ~3-35 MB regardless of file size! 🚀

### Mining Different Fields (New in v0.2.1!)

**No preprocessing needed!** Use `ColumnMapping` to mine any fields directly:

```rust
use rust_rule_miner::data_loader::{DataLoader, ColumnMapping};

// CSV: customer_id, product_name, category, price, location, timestamp
//      0            1             2         3      4         5

// Option 1: Mine product names (column 1)
let mapping = ColumnMapping::simple(0, 1, 5);
let transactions = DataLoader::from_csv_with_mapping("sales.csv", mapping)?;

// Option 2: Mine categories (column 2)
let mapping = ColumnMapping::simple(0, 2, 5);
let transactions = DataLoader::from_csv_with_mapping("sales.csv", mapping)?;

// Option 3: Mine product + category combined
let mapping = ColumnMapping::multi_field(
    0,                  // transaction_id column
    vec![1, 2],         // product(1) + category(2)
    5,                  // timestamp column
    "::".to_string()    // separator
);
let transactions = DataLoader::from_csv_with_mapping("sales.csv", mapping)?;
// Items: "Laptop::Electronics", "Mouse::Accessories"

// Option 4: Mine product + category + location
let mapping = ColumnMapping::multi_field(0, vec![1, 2, 4], 5, "::".to_string());
let transactions = DataLoader::from_csv_with_mapping("sales.csv", mapping)?;
// Items: "Laptop::Electronics::US", "Mouse::Accessories::UK"
```

**Multi-field zipping:** If your CSV has comma-separated values in multiple columns:

```csv
customer_id,products,categories,locations,timestamp
123,"Laptop,Mouse","Electronics,Accessories","US,US",2024-01-01
```

The miner will automatically zip them together:
```rust
let mapping = ColumnMapping::multi_field(0, vec![1, 2, 3], 4, "::".to_string());
// Result: ["Laptop::Electronics::US", "Mouse::Accessories::US"]
```

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

## 🎨 Engine Integration (New in v0.2!)

Execute mined rules in real-time with integrated engine support.

**Two-Phase Approach:**
1. **Mining Phase**: Apply quality criteria (min_support, min_confidence, min_lift) to filter rules
2. **Execution Phase**: Execute pre-filtered high-quality rules in real-time

```rust
use rust_rule_miner::engine::{MiningRuleEngine, facts_from_cart};

// PHASE 1: Mine rules with quality criteria
let config = MiningConfig {
    min_support: 0.3,       // Pattern must appear in 30%+ of transactions
    min_confidence: 0.7,    // Rule must be correct 70%+ of the time
    min_lift: 1.2,          // Rule must be 20%+ better than random
    ..Default::default()
};

let mut miner = RuleMiner::new(config);
miner.add_transactions(transactions)?;
let rules = miner.mine_association_rules()?;  // ← Only high-quality rules

// PHASE 2: Load filtered rules into engine and execute
let mut engine = MiningRuleEngine::new("ProductRecommendations");
engine.load_rules(&rules)?;  // ← Loads only the filtered rules from Phase 1

// Execute in real-time
let facts = facts_from_cart(vec!["Laptop".to_string()]);
let result = engine.execute(&facts)?;

if let Some(recommendations) = result.get("Recommendation.items") {
    println!("Recommend: {:?}", recommendations);  // ["Mouse", "Keyboard"]
}
```

**Key Point**: Mining criteria are applied during `mine_association_rules()`, not during execution. The engine only executes pre-filtered high-quality rules.

### Flexible GRL Export for Any Domain

No more hardcoded field names! Configure for any use case:

```rust
use rust_rule_miner::export::GrlConfig;

// E-commerce
let config = GrlConfig::custom("Cart.items", "Recommendations.products");

// Fraud detection
let config = GrlConfig::custom("Transaction.indicators", "FraudAlert.flags");

// Security
let config = GrlConfig::custom("NetworkActivity.events", "SecurityAlert.threats");

// Generate GRL with custom fields
let grl = GrlExporter::to_grl_with_config(&rules, &config);
```

See [examples/flexible_domain_mining.rs](examples/flexible_domain_mining.rs) for complete examples across multiple domains.

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

**Basic Examples:**
- `01_simple_ecommerce.rs` - Simple e-commerce association rule mining
- `02_medium_complexity.rs` - Medium complexity patterns
- `03_advanced_large_dataset.rs` - Advanced mining with large datasets
- `04_load_from_excel_csv.rs` - Loading data from Excel/CSV files
- `basic_mining.rs` - Basic association rule mining

**Engine Integration (v0.2.0+):**
- `integration_with_engine.rs` - Native engine integration (simple API)
- `integration_with_rete.rs` - RETE engine for high performance
- `flexible_domain_mining.rs` - Multi-domain examples (fraud, security, content)
- `postgres_stream_mining.rs` - PostgreSQL streaming + mining

**Advanced:**
- `performance_test.rs` - Performance benchmarking
- `cloud_demo.rs` - Cloud storage integration (S3, HTTP)
- `excelstream_demo.rs` - Excel streaming examples

---

## 🗺️ Roadmap

**Completed (v0.2.0):**
- [x] Apriori algorithm
- [x] Association rule generation
- [x] Quality metrics (confidence, support, lift)
- [x] GRL export with flexible field configuration
- [x] Engine integration (rust-rule-engine)
- [x] PostgreSQL streaming support
- [x] Multi-domain support (e-commerce, fraud, security)
- [x] Excel/CSV data loading
- [x] **Column mapping configuration** - Select and combine fields from multi-column data

**Planned:**
- [ ] FP-Growth algorithm optimization
- [ ] Sequential pattern mining
- [ ] Graph pattern matching
- [ ] Incremental mining (update rules with new data)
- [ ] Multi-level mining (category hierarchies)
- [ ] Negative pattern mining
- [ ] Real-time streaming with LISTEN/NOTIFY
- [ ] Rule versioning and A/B testing
- [ ] WebAssembly support

---

## 📖 Documentation

**Getting Started:**
- [API Documentation](https://docs.rs/rust-rule-miner)
- [Getting Started Guide](docs/GETTING_STARTED.md)
- [User Guide](docs/USER_GUIDE.md)

**v0.2.0 Engine Integration:**
- [Engine Integration Summary](ENGINE_INTEGRATION_SUMMARY.md) - Complete v0.2.0 overview
- [Integration Guide](INTEGRATION_GUIDE.md) - Detailed API guide with examples
- [PostgreSQL Streaming](examples/POSTGRES_STREAMING.md) - Database integration tutorial

**Advanced Topics:**
- [Integration with Web Frameworks](docs/INTEGRATION.md) - Actix-Web, Axum, production deployment
- [Performance Tuning](docs/PERFORMANCE.md) - Benchmarks and optimization
- [Algorithm Details](docs/ALGORITHMS.md) - Technical algorithm documentation
- [Advanced Usage](docs/ADVANCED.md)

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
