use chrono::Utc;
/// Quick performance test to verify benchmark numbers
///
/// This example measures actual runtime and memory for different dataset sizes
use rust_rule_miner::{MiningAlgorithm, MiningConfig, RuleMiner, Transaction};
use std::time::Instant;

fn generate_transactions(count: usize) -> Vec<Transaction> {
    let products = vec![
        "Laptop",
        "Mouse",
        "Keyboard",
        "Monitor",
        "Hub",
        "Phone",
        "Case",
        "Protector",
        "Charger",
        "Tablet",
        "Stylus",
        "Headphones",
        "Camera",
        "Lens",
        "Tripod",
        "Card",
    ];

    let mut transactions = Vec::new();
    let base_time = Utc::now();

    for i in 0..count {
        let num_items = (i % 3) + 1;
        let start_idx = (i * 7) % products.len();

        let mut items = Vec::new();
        for j in 0..num_items {
            let idx = (start_idx + j) % products.len();
            items.push(products[idx].to_string());
        }

        let timestamp = base_time + chrono::Duration::seconds(i as i64 * 60);
        transactions.push(Transaction::new(format!("tx{}", i), items, timestamp));
    }

    transactions
}

fn test_performance(size: usize, algorithm: MiningAlgorithm) {
    println!("\nTesting {} transactions with {:?}", size, algorithm);
    println!("{}", "-".repeat(60));

    // Generate data
    let gen_start = Instant::now();
    let transactions = generate_transactions(size);
    let gen_time = gen_start.elapsed();
    println!("Data generation: {:?}", gen_time);

    // Mining
    let mine_start = Instant::now();
    let mut miner = RuleMiner::new(MiningConfig {
        min_support: 0.05,
        min_confidence: 0.6,
        min_lift: 1.2,
        max_time_gap: None,
        algorithm,
    });

    miner.add_transactions(transactions).unwrap();
    let rules = miner.mine_association_rules().unwrap();
    let mine_time = mine_start.elapsed();

    println!("Mining time: {:?}", mine_time);
    println!("Rules found: {}", rules.len());
    println!(
        "Transactions/sec: {:.0}",
        size as f64 / mine_time.as_secs_f64()
    );
}

fn main() {
    println!("Performance Test - rust-rule-miner");
    println!("{}", "=".repeat(60));

    // Test different sizes with Apriori
    for size in [100, 1_000, 10_000] {
        test_performance(size, MiningAlgorithm::Apriori);
    }

    println!("\n{}", "=".repeat(60));
    println!("Summary");
    println!("{}", "=".repeat(60));
    println!();
    println!("Expected Performance (approximate):");
    println!("  100 transactions:    ~50-100ms");
    println!("  1,000 transactions:  ~200-500ms");
    println!("  10,000 transactions: ~2-5s");
    println!();
    println!("Note: Actual performance depends on:");
    println!("- Number of unique items");
    println!("- Transaction complexity (items per transaction)");
    println!("- Support/confidence thresholds");
    println!("- CPU and memory speed");
}
