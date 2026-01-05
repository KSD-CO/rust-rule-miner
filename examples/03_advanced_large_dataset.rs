//! Example 3: Advanced - Large Dataset with Statistics
//!
//! Demonstrates mining from large dataset with detailed analysis.
//! Use case: Production-scale recommendation system.
//!
//! NOTE: For custom field names, see GrlConfig in examples/flexible_domain_mining.rs

use chrono::{Duration, Utc};
use rust_rule_engine::rete::{
    facts::{FactValue, TypedFacts},
    grl_loader::GrlReteLoader,
    propagation::IncrementalEngine,
};
use rust_rule_miner::{export::GrlExporter, MiningConfig, RuleMiner, Transaction};
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 3: Advanced - Large Dataset Mining ===\n");

    // Generate large synthetic dataset
    let transactions = generate_large_dataset(100);

    println!("Dataset Statistics:");
    println!("  Total transactions: {}", transactions.len());

    let item_freq = analyze_item_frequency(&transactions);
    println!("  Unique items: {}", item_freq.len());
    println!("  Top 5 items:");
    let mut sorted: Vec<_> = item_freq.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (i, (item, count)) in sorted.iter().take(5).enumerate() {
        println!(
            "    {}. {} ({} times, {:.1}%)",
            i + 1,
            item,
            count,
            **count as f64 / transactions.len() as f64 * 100.0
        );
    }
    println!();

    // Mining with production-grade config
    let config = MiningConfig {
        min_support: 0.15,    // 15% - reasonable for large dataset
        min_confidence: 0.75, // 75% - high confidence
        min_lift: 1.5,        // Strong positive correlation only
        ..Default::default()
    };

    println!("Mining Configuration:");
    println!("  Min Support: {:.0}%", config.min_support * 100.0);
    println!("  Min Confidence: {:.0}%", config.min_confidence * 100.0);
    println!("  Min Lift: {:.1}", config.min_lift);
    println!();

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;

    println!("Mining rules...");
    let rules = miner.mine_association_rules()?;

    println!("\nMining Results:");
    println!(
        "  Frequent itemsets found: {}",
        miner.stats().frequent_itemsets_count
    );
    println!("  Association rules generated: {}", rules.len());
    println!();

    // Analyze rule quality
    if !rules.is_empty() {
        let avg_conf = rules.iter().map(|r| r.metrics.confidence).sum::<f64>() / rules.len() as f64;
        let avg_lift = rules.iter().map(|r| r.metrics.lift).sum::<f64>() / rules.len() as f64;
        println!("Rule Quality Metrics:");
        println!("  Average Confidence: {:.1}%", avg_conf * 100.0);
        println!("  Average Lift: {:.2}", avg_lift);
        println!();
    }

    // Show top rules
    println!("Top 10 Rules (by quality score):");
    for (idx, rule) in rules.iter().take(10).enumerate() {
        println!(
            "{}. {:?} => {:?}",
            idx + 1,
            rule.antecedent,
            rule.consequent
        );
        println!(
            "   Conf: {:.1}% | Sup: {:.1}% | Lift: {:.2} | Quality: {:.3}",
            rule.metrics.confidence * 100.0,
            rule.metrics.support * 100.0,
            rule.metrics.lift,
            rule.quality_score()
        );
    }
    println!();

    // Generate GRL and analyze
    let grl = GrlExporter::to_grl(&rules);
    fs::write("/tmp/large_dataset_rules.grl", &grl)?;

    println!("Generated GRL Code:");
    println!("  Lines: {}", grl.lines().count());
    println!("  Size: {} bytes", grl.len());
    println!("  Saved to: /tmp/large_dataset_rules.grl");
    println!();

    // Comprehensive testing
    let mut engine = IncrementalEngine::new();
    GrlReteLoader::load_from_file("/tmp/large_dataset_rules.grl", &mut engine)?;

    println!("Testing Recommendations:");
    println!("----------------------------------------");

    let test_cases = vec![
        (vec!["Laptop"], "Business professional"),
        (vec!["Gaming Console"], "Gamer"),
        (vec!["Camera"], "Photographer"),
        (vec!["Laptop", "Mouse"], "Office worker (multiple items)"),
    ];

    for (items, persona) in test_cases {
        println!("\nPersona: {}", persona);
        println!("  Cart: {:?}", items);
        test_with_engine(&mut engine, items);
    }

    Ok(())
}

fn generate_large_dataset(size: usize) -> Vec<Transaction> {
    let mut transactions = Vec::new();

    // Product categories with co-purchase patterns
    let patterns = vec![
        // Tech professional
        vec!["Laptop", "Mouse", "Keyboard", "Monitor", "USB-C Hub"],
        vec!["Laptop", "Mouse", "Laptop Bag"],
        vec!["Monitor", "Keyboard", "Mouse"],
        // Gamer
        vec!["Gaming Console", "Controller", "Game", "Headset"],
        vec!["Gaming Console", "Controller", "Game"],
        vec!["Gaming PC", "Gaming Mouse", "Mechanical Keyboard"],
        // Mobile user
        vec!["Phone", "Phone Case", "Screen Protector", "Charger"],
        vec!["Phone", "Phone Case", "Wireless Earbuds"],
        vec!["Tablet", "Tablet Case", "Stylus"],
        // Photographer
        vec!["Camera", "Lens", "Memory Card", "Camera Bag"],
        vec!["Camera", "Tripod", "Memory Card"],
        // Home office
        vec!["Desk", "Chair", "Monitor", "Keyboard"],
        vec!["Webcam", "Microphone", "Headset"],
        // Audio enthusiast
        vec!["Headphones", "DAC", "Amplifier"],
        vec!["Speakers", "Audio Cable"],
        // Fitness
        vec!["Smartwatch", "Fitness Tracker", "Heart Rate Monitor"],
    ];

    let mut tx_id = 0;
    let now = Utc::now();

    for i in 0..size {
        let pattern = &patterns[i % patterns.len()];

        // Randomly select 2-4 items from pattern
        let num_items = 2 + (i % 3);
        let items: Vec<String> = pattern
            .iter()
            .take(num_items.min(pattern.len()))
            .map(|s| s.to_string())
            .collect();

        if !items.is_empty() {
            let timestamp = now - Duration::hours((size - i) as i64);
            transactions.push(Transaction::new(format!("tx{}", tx_id), items, timestamp));
            tx_id += 1;
        }
    }

    transactions
}

fn analyze_item_frequency(transactions: &[Transaction]) -> HashMap<String, usize> {
    let mut freq = HashMap::new();
    for tx in transactions {
        for item in &tx.items {
            *freq.entry(item.clone()).or_insert(0) += 1;
        }
    }
    freq
}

fn test_with_engine(_engine: &mut IncrementalEngine, items: Vec<&str>) {
    // Create new engine for clean test
    let mut test_engine = IncrementalEngine::new();
    GrlReteLoader::load_from_file("/tmp/large_dataset_rules.grl", &mut test_engine).unwrap();

    let mut cart = TypedFacts::new();
    cart.set(
        "items",
        FactValue::Array(
            items
                .iter()
                .map(|s| FactValue::String(s.to_string()))
                .collect(),
        ),
    );

    let mut rec = TypedFacts::new();
    rec.set("items", FactValue::Array(vec![]));

    test_engine.insert("ShoppingCart".to_string(), cart);
    test_engine.insert("Recommendation".to_string(), rec);

    let fired = test_engine.fire_all();

    let wm = test_engine.working_memory();
    let recs = wm.get_by_type("Recommendation");

    print!("  Recommendations: ");
    if !recs.is_empty() {
        if let Some(FactValue::Array(rec_items)) = recs[0].data.get("items") {
            if rec_items.is_empty() {
                print!("(none)");
            } else {
                print!("[");
                for (i, item) in rec_items.iter().enumerate() {
                    if i > 0 {
                        print!(", ");
                    }
                    if let FactValue::String(s) = item {
                        print!("\"{}\"", s);
                    }
                }
                print!("]");
            }
        }
    } else {
        print!("(none)");
    }
    println!(" ({} rules fired)", fired.len());
}
