# Performance Guide

## Overview

This guide provides performance characteristics, optimization tips, and benchmarking data for rust-rule-miner.

## Benchmark Results

### Apriori Algorithm

| Dataset Size | Time (avg) | Memory (peak) | Rules Found | Throughput |
|--------------|------------|---------------|-------------|------------|
| 100 txs      | ~10-20ms   | ~5 MB         | 5-10        | 5,000-10,000 tx/s |
| 1,000 txs    | ~100-200ms | ~10-15 MB     | 20-50       | 5,000-10,000 tx/s |
| 10,000 txs   | ~1-2s      | ~30-50 MB     | 50-200      | 5,000-10,000 tx/s |
| 100,000 txs  | ~10-20s    | ~200-500 MB   | 200-1000    | 5,000-10,000 tx/s |

**Configuration:**
```rust
MiningConfig {
    min_support: 0.05,      // 5%
    min_confidence: 0.6,    // 60%
    min_lift: 1.2,
    algorithm: MiningAlgorithm::Apriori,
}
```

### Memory Usage

Memory usage is primarily determined by:
1. **Number of unique items**: More items = more itemset combinations
2. **Transaction complexity**: More items per transaction = more patterns
3. **Support threshold**: Lower support = more frequent itemsets

**Formula (approximate):**
```
Peak Memory ≈ (num_transactions × avg_items) + (num_frequent_itemsets × itemset_size)
```

### Performance Factors

#### 1. Support Threshold

Lower support = More itemsets = Higher memory & time:

| min_support | 1K txs | 10K txs | 100K txs |
|-------------|--------|---------|----------|
| 0.01 (1%)   | ~200ms | ~3s     | ~30s     |
| 0.05 (5%)   | ~100ms | ~1.5s   | ~15s     |
| 0.10 (10%)  | ~50ms  | ~800ms  | ~8s      |
| 0.20 (20%)  | ~30ms  | ~400ms  | ~4s      |

#### 2. Confidence Threshold

Confidence affects rule generation, not itemset mining:

| min_confidence | Rules Generated | Post-processing Time |
|----------------|-----------------|---------------------|
| 0.3 (30%)      | ~500-1000       | +200ms              |
| 0.6 (60%)      | ~100-300        | +50ms               |
| 0.8 (80%)      | ~50-100         | +20ms               |
| 0.95 (95%)     | ~10-30          | +5ms                |

#### 3. Number of Unique Items

| Unique Items | Combinations | Impact |
|--------------|--------------|--------|
| 10           | Low          | Fast   |
| 50           | Medium       | Moderate |
| 100          | High         | Slower |
| 500+         | Very High    | Much slower |

**Recommendation**: If you have >100 unique items, consider:
- Grouping items into categories
- Increasing min_support threshold
- Using multi-level mining

## Optimization Strategies

### 1. Tune Thresholds

Start conservative, then relax:

```rust
// Start here
let config = MiningConfig {
    min_support: 0.1,       // 10%
    min_confidence: 0.8,    // 80%
    min_lift: 2.0,
    ..Default::default()
};

// If too few rules, gradually decrease
let config = MiningConfig {
    min_support: 0.05,      // 5%
    min_confidence: 0.6,    // 60%
    min_lift: 1.5,
    ..Default::default()
};
```

### 2. Batch Processing

For very large datasets, process in batches:

```rust
const BATCH_SIZE: usize = 50_000;

let all_transactions = DataLoader::from_csv("huge_file.csv")?;

for (i, chunk) in all_transactions.chunks(BATCH_SIZE).enumerate() {
    println!("Processing batch {}", i + 1);

    let mut miner = RuleMiner::new(config.clone());
    miner.add_transactions(chunk.to_vec())?;
    let batch_rules = miner.mine_association_rules()?;

    // Merge rules...
}
```

### 3. Parallel Processing

Process multiple datasets concurrently:

```rust
use rayon::prelude::*;

let files = vec!["data1.csv", "data2.csv", "data3.csv"];

let all_rules: Vec<_> = files.par_iter()
    .filter_map(|file| {
        let transactions = DataLoader::from_csv(file).ok()?;
        let mut miner = RuleMiner::new(config.clone());
        miner.add_transactions(transactions).ok()?;
        miner.mine_association_rules().ok()
    })
    .flatten()
    .collect();
```

### 4. Data Preprocessing

Clean data before mining:

```rust
// Remove duplicates
let mut seen = HashSet::new();
let unique: Vec<_> = transactions.into_iter()
    .filter(|tx| seen.insert(tx.id.clone()))
    .collect();

// Remove empty transactions
let filtered: Vec<_> = transactions.into_iter()
    .filter(|tx| !tx.items.is_empty())
    .collect();

// Normalize item names
let normalized: Vec<_> = transactions.into_iter()
    .map(|mut tx| {
        tx.items = tx.items.iter()
            .map(|item| item.trim().to_lowercase())
            .collect();
        tx
    })
    .collect();
```

## Memory Management

### excelstream Integration

excelstream provides constant memory usage regardless of file size:

```rust
// Traditional: Memory = File size
let transactions = std::fs::read_to_string("huge_file.csv")?
    .lines()
    .map(|line| parse_line(line))
    .collect();  // Uses ~1GB RAM for 1GB file

// excelstream: Constant ~3-35 MB
let transactions = DataLoader::from_csv("huge_file.csv")?;  // Only ~3-35 MB!
```

### Memory Profiling

Monitor memory during mining:

```rust
use std::time::Instant;

let start = Instant::now();
let mut miner = RuleMiner::new(config);

println!("Before mining: Check memory usage");
miner.add_transactions(transactions)?;

println!("After adding transactions: Check memory");
let rules = miner.mine_association_rules()?;

println!("After mining: Check memory");
println!("Time elapsed: {:?}", start.elapsed());
println!("Rules generated: {}", rules.len());
```

## Bottleneck Analysis

### Time Breakdown

Typical time distribution for 10K transactions:

```
Data Loading:         10%   (~100ms with excelstream)
Itemset Mining:       70%   (~700ms Apriori algorithm)
Rule Generation:      15%   (~150ms)
Post-processing:      5%    (~50ms filtering, sorting)
```

### Optimization Priority

1. **Itemset Mining** (70% of time)
   - Increase min_support to reduce candidates
   - Use FP-Growth for large datasets (future)

2. **Data Loading** (10% of time)
   - Use excelstream (already optimized)
   - Preprocess data offline

3. **Rule Generation** (15% of time)
   - Increase min_confidence
   - Filter bidirectional rules (already done)

4. **Post-processing** (5% of time)
   - Limit max rules returned
   - Sort only top N rules

## Scaling Guidelines

### Small Scale (< 10K transactions)

- **Algorithm**: Apriori
- **Config**: Default thresholds
- **Memory**: <100 MB
- **Time**: < 5 seconds

```rust
let config = MiningConfig::default();
```

### Medium Scale (10K - 100K transactions)

- **Algorithm**: Apriori
- **Config**: Higher thresholds
- **Memory**: 100-500 MB
- **Time**: 5-30 seconds

```rust
let config = MiningConfig {
    min_support: 0.05,
    min_confidence: 0.7,
    min_lift: 1.5,
    ..Default::default()
};
```

### Large Scale (100K - 1M transactions)

- **Algorithm**: FP-Growth (when implemented)
- **Config**: Conservative thresholds
- **Memory**: 500 MB - 2 GB
- **Time**: 30s - 5 minutes

```rust
let config = MiningConfig {
    min_support: 0.1,
    min_confidence: 0.8,
    min_lift: 2.0,
    algorithm: MiningAlgorithm::Apriori,  // FPGrowth coming soon
};
```

### Very Large Scale (1M+ transactions)

- **Strategy**: Batch processing or sampling
- **Cloud**: Use S3 streaming with serverless
- **Memory**: Distributed across workers

```rust
// Sample approach
let sample_size = 100_000;
let sampled: Vec<_> = transactions
    .into_iter()
    .step_by(transactions.len() / sample_size)
    .collect();
```

## Running Benchmarks

### Quick Performance Test

```bash
cargo run --example performance_test --release
```

### Full Benchmark Suite

```bash
cargo bench --bench performance_benchmark
```

### Custom Benchmark

```rust
use std::time::Instant;

let start = Instant::now();

// Your mining code here
let mut miner = RuleMiner::new(config);
miner.add_transactions(transactions)?;
let rules = miner.mine_association_rules()?;

let elapsed = start.elapsed();
println!("Time: {:?}", elapsed);
println!("Throughput: {:.0} tx/s",
    transactions.len() as f64 / elapsed.as_secs_f64());
```

## Performance Tips Summary

✅ **DO:**
- Start with high thresholds, then decrease
- Use excelstream for large files
- Preprocess data (remove duplicates, normalize)
- Monitor memory and time during development
- Batch process for very large datasets
- Use release builds for accurate measurements

❌ **DON'T:**
- Set min_support < 0.01 without good reason
- Load huge files into memory (use excelstream)
- Mine with debug builds (10x slower)
- Process raw data without cleaning
- Ignore memory constraints

## Comparison with Other Tools

| Tool | Language | 10K txs | 100K txs | Memory |
|------|----------|---------|----------|--------|
| rust-rule-miner | Rust | ~1-2s | ~10-20s | 30-500 MB |
| mlxtend (Python) | Python | ~5-10s | ~60-120s | 500 MB - 2 GB |
| arules (R) | R | ~3-8s | ~40-100s | 300 MB - 1 GB |
| Weka | Java | ~4-10s | ~50-120s | 400 MB - 1.5 GB |

*Note: Benchmarks are approximate and depend on configuration and hardware*

## Hardware Recommendations

### Minimum

- **CPU**: 2 cores, 2.0 GHz
- **RAM**: 4 GB
- **Disk**: HDD
- **Suitable for**: <10K transactions

### Recommended

- **CPU**: 4 cores, 2.5 GHz+
- **RAM**: 8-16 GB
- **Disk**: SSD
- **Suitable for**: <100K transactions

### High Performance

- **CPU**: 8+ cores, 3.0 GHz+
- **RAM**: 32+ GB
- **Disk**: NVMe SSD
- **Suitable for**: 1M+ transactions

---

**Related Documentation:**
- [Getting Started](GETTING_STARTED.md)
- [excelstream Integration](EXCELSTREAM.md)
- [Advanced Topics](ADVANCED.md)
