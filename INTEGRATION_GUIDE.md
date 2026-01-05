# rust-rule-engine Core Integration Guide

## Overview

The project has been enhanced with:
1. ✅ **rust-rule-engine integrated into core** (no longer just a dev-dependency)
2. ✅ **New engine module** in `src/engine/mod.rs` for direct rule execution
3. ✅ **Flexible GRL Export** - no more hardcoded `ShoppingCart.items`
4. ✅ **PostgreSQL streaming support** - stream data from database and mine rules in real-time

## Major Changes

### 1. Cargo.toml - New Dependencies

```toml
[features]
default = []
cloud = ["tokio", "reqwest"]  # Cloud storage support
engine = ["rust-rule-engine"]  # Rule engine integration 🆕
postgres = ["tokio-postgres", "bb8", "bb8-postgres", "tokio"]  # PostgreSQL support 🆕

[dependencies]
# Rule engine integration (core feature) 🆕
rust-rule-engine = { version = "1.15.0", optional = true }

# PostgreSQL support for streaming data 🆕
tokio-postgres = { version = "0.7", features = ["with-chrono-0_4"], optional = true }
bb8 = { version = "0.8", optional = true }
bb8-postgres = { version = "0.8", optional = true }
```

### 2. Module src/engine/mod.rs - Engine Integration 🆕

New module providing simple API for rule execution:

```rust
use rust_rule_miner::engine::{MiningRuleEngine, facts_from_cart};

// Create engine
let mut engine = MiningRuleEngine::new("MyRules");

// Load mined rules
engine.load_rules(&rules)?;

// Execute with facts
let facts = facts_from_cart(vec!["Laptop".to_string()]);
let result = engine.execute(&facts)?;

println!("Rules fired: {}", result.rules_fired);
if let Some(recommendations) = result.get("Recommendation.items") {
    println!("Recommendations: {:?}", recommendations);
}
```

#### Main API:

- `MiningRuleEngine::new(name)` - Create new engine
- `MiningRuleEngine::with_config(name, grl_config)` - Create with custom GRL config
- `load_rules(&rules)` - Load association rules into engine
- `execute(&facts)` - Execute rules and return results
- `facts_from_cart(items)` - Helper to create Facts for shopping cart
- `facts_from_transaction(items)` - Helper to create Facts for transaction

### 3. GRL Export - No More Hardcoding 🎉

#### Old Problem:
```rust
// ❌ Hardcoded "ShoppingCart.items" and "Recommendation.items"
let grl = GrlExporter::to_grl(&rules);
```

#### New Solution:
```rust
use rust_rule_miner::export::GrlConfig;

// ✅ Option 1: Use default (ShoppingCart.items, Recommendation.items)
let grl = GrlExporter::to_grl(&rules);

// ✅ Option 2: Custom fields
let config = GrlConfig::new("Transaction.items", "Analysis.recommendations");
let grl = GrlExporter::to_grl_with_config(&rules, &config);

// ✅ Option 3: Preset configs
let config = GrlConfig::transaction(); // Transaction.items → Analysis.recommendations
let config = GrlConfig::shopping_cart(); // ShoppingCart.items → Recommendation.items
let config = GrlConfig::custom("Order.products", "Upsell.suggestions");
```

### 4. PostgreSQL Streaming Example 🆕

File: [`examples/postgres_stream_mining.rs`](examples/postgres_stream_mining.rs)

Complete workflow:
```
PostgreSQL → Stream Data → Mine Rules → Load Engine → Real-time Execution
```

## Usage

### A. Engine Integration (Basic)

```rust
use rust_rule_miner::{
    MiningConfig, RuleMiner, Transaction,
    engine::MiningRuleEngine,
};
use chrono::Utc;

// 1. Mine rules from data
let transactions = vec![
    Transaction::new("tx1", vec!["A".into(), "B".into()], Utc::now()),
    Transaction::new("tx2", vec!["A".into(), "B".into()], Utc::now()),
];

let config = MiningConfig {
    min_support: 0.5,
    min_confidence: 0.7,
    ..Default::default()
};

let mut miner = RuleMiner::new(config);
miner.add_transactions(transactions)?;
let rules = miner.mine_association_rules()?;

// 2. Load into engine
let mut engine = MiningRuleEngine::new("MyRules");
engine.load_rules(&rules)?;

// 3. Execute in real-time
let facts = facts_from_cart(vec!["A".to_string()]);
let result = engine.execute(&facts)?;
```

### B. Custom GRL Fields

```rust
use rust_rule_miner::export::GrlConfig;

// Create engine with custom fields
let config = GrlConfig::custom(
    "Order.items",
    "CrossSell.products"
);

let mut engine = MiningRuleEngine::with_config("MyRules", config);
engine.load_rules(&rules)?;
```

### C. PostgreSQL Streaming

```bash
# 1. Setup database
createdb rule_mining_demo
psql rule_mining_demo < examples/postgres_setup.sql

# 2. Set DATABASE_URL
export DATABASE_URL="postgresql://user:pass@localhost/rule_mining_demo"

# 3. Run example
cargo run --example postgres_stream_mining --features "postgres,engine"
```

## Comparison: Native vs RETE Engine

Currently using **RustRuleEngine** (native). For RETE engine:

### When to use Native (RustRuleEngine)?
- ✅ Few rules (< 100 rules)
- ✅ Simple use cases
- ✅ Simple API, easy to use
- ✅ Already integrated in MiningRuleEngine

### When to use RETE (IncrementalEngine)?
- ✅ Many rules (> 100 rules)
- ✅ High performance requirements
- ✅ Complex pattern matching
- ✅ See example: `examples/integration_with_rete.rs`

```rust
// RETE engine (for advanced use)
use rust_rule_engine::rete::propagation::IncrementalEngine;

let mut engine = IncrementalEngine::new();
GrlReteLoader::load_from_file("rules.grl", &mut engine)?;
```

## Complete Examples

### 1. Basic Integration
```bash
cargo run --example integration_with_engine --features "engine"
```

### 2. RETE Engine
```bash
cargo run --example integration_with_rete --features "engine"
```

### 3. PostgreSQL Streaming
```bash
cargo run --example postgres_stream_mining --features "postgres,engine"
```

## Code Structure

```
rust-rule-miner/
├── src/
│   ├── engine/           # 🆕 Engine integration
│   │   └── mod.rs        # MiningRuleEngine, facts helpers
│   ├── export/
│   │   ├── mod.rs
│   │   └── grl.rs        # 🔧 Updated: GrlConfig, flexible export
│   ├── mining/           # Mining algorithms
│   ├── lib.rs            # 🔧 Updated: export engine module
│   └── ...
├── examples/
│   ├── postgres_stream_mining.rs  # 🆕 PostgreSQL streaming
│   ├── postgres_setup.sql          # 🆕 SQL schema & data
│   ├── integration_with_engine.rs  # Native engine
│   ├── integration_with_rete.rs    # RETE engine
│   └── ...
└── Cargo.toml            # 🔧 Updated: new features & deps
```

## Best Practices

### 1. Choose Engine Type
```rust
// Development/Testing: Native (simple)
let mut engine = MiningRuleEngine::new("Dev");

// Production with many rules: Use RETE directly
// (See examples/integration_with_rete.rs)
```

### 2. Custom Fields for Specific Domains
```rust
// E-commerce
let config = GrlConfig::custom("Cart.items", "Recommendation.products");

// Analytics
let config = GrlConfig::custom("Event.tags", "Prediction.categories");

// Fraud detection
let config = GrlConfig::custom("Transaction.features", "Risk.alerts");
```

### 3. Error Handling
```rust
match engine.load_rules(&rules) {
    Ok(count) => println!("Loaded {} rules", count),
    Err(e) => eprintln!("Failed to load rules: {}", e),
}
```

## Performance Tips

1. **Batch Loading**: Load all rules at once instead of one by one
2. **Connection Pooling**: Use `bb8` for PostgreSQL in production
3. **Caching**: Cache mined rules to avoid re-mining
4. **Monitoring**: Track `rules_fired` to optimize mining parameters

## Roadmap

- [ ] Support RETE engine in MiningRuleEngine (wrapper API)
- [ ] Real-time streaming with PostgreSQL LISTEN/NOTIFY
- [ ] Incremental rule mining when new data arrives
- [ ] Rule versioning and A/B testing
- [ ] Metrics and monitoring dashboard

## Use Case: Different Domains

### Example: Fraud Detection System

```rust
use rust_rule_miner::{
    MiningConfig, RuleMiner, Transaction,
    engine::{MiningRuleEngine, facts_from_items},
    export::GrlConfig,
};

// 1. Define custom fields for domain
let config = GrlConfig::custom(
    "Transaction.indicators",  // Input
    "FraudAlert.flags"         // Output
);

// 2. Mine rules from historical fraud data
let fraud_patterns = vec![
    Transaction::new("f1", vec!["Multiple_IPs".into(), "Large_Amount".into()], Utc::now()),
    // ... more patterns
];

let mut miner = RuleMiner::new(MiningConfig::default());
miner.add_transactions(fraud_patterns)?;
let rules = miner.mine_association_rules()?;

// 3. Setup engine
let mut engine = MiningRuleEngine::with_config("FraudDetection", config.clone());
engine.load_rules(&rules)?;

// 4. Analyze new transaction
let indicators = vec!["Multiple_IPs".to_string()];
let facts = facts_from_items(indicators, &config);
let result = engine.execute(&facts)?;

if let Some(alerts) = result.get("FraudAlert.flags") {
    println!("⚠️ Fraud detected: {:?}", alerts);
}
```

## Documentation

- [INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md) - Complete guide
- [POSTGRES_STREAMING.md](examples/POSTGRES_STREAMING.md) - PostgreSQL guide
- [Engine API docs](src/engine/mod.rs) - Module documentation

## Key Benefits

1. **No More Hardcoding** - Works for any domain
2. **Type-safe** - Compile-time checks with Rust
3. **Performance** - RETE algorithm ready for production
4. **Flexible** - Custom fields for any use case
5. **Real-time** - Execute rules immediately

Everything is ready to use! 🚀
