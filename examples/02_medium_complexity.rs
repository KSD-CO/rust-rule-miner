//! Example 2: Medium Complexity - Multiple Product Categories
//!
//! Demonstrates mining with more diverse dataset.
//! Use case: Cross-category recommendations.

use chrono::Utc;
use rust_rule_engine::rete::{
    facts::{FactValue, TypedFacts},
    grl_loader::GrlReteLoader,
    propagation::IncrementalEngine,
};
use rust_rule_miner::{export::GrlExporter, MiningConfig, RuleMiner, Transaction};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 2: Medium Complexity - Cross-Category Recommendations ===\n");

    // Medium dataset: 15 transactions across different categories
    let transactions = vec![
        // Electronics bundle
        Transaction::new(
            "tx1",
            vec![
                "Laptop".to_string(),
                "Mouse".to_string(),
                "USB-C Hub".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "tx2",
            vec![
                "Laptop".to_string(),
                "Mouse".to_string(),
                "Laptop Bag".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "tx3",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx4",
            vec!["Laptop".to_string(), "USB-C Hub".to_string()],
            Utc::now(),
        ),
        // Mobile accessories
        Transaction::new(
            "tx5",
            vec![
                "Phone".to_string(),
                "Phone Case".to_string(),
                "Screen Protector".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "tx6",
            vec![
                "Phone".to_string(),
                "Phone Case".to_string(),
                "Charger".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "tx7",
            vec!["Phone".to_string(), "Phone Case".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx8",
            vec!["Phone".to_string(), "Screen Protector".to_string()],
            Utc::now(),
        ),
        // Gaming
        Transaction::new(
            "tx9",
            vec![
                "Gaming Console".to_string(),
                "Controller".to_string(),
                "Game".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "tx10",
            vec!["Gaming Console".to_string(), "Controller".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx11",
            vec!["Gaming Console".to_string(), "Game".to_string()],
            Utc::now(),
        ),
        // Office
        Transaction::new(
            "tx12",
            vec![
                "Monitor".to_string(),
                "Keyboard".to_string(),
                "Mouse".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "tx13",
            vec!["Monitor".to_string(), "Keyboard".to_string()],
            Utc::now(),
        ),
        // Mixed
        Transaction::new(
            "tx14",
            vec!["Tablet".to_string(), "Stylus".to_string()],
            Utc::now(),
        ),
        Transaction::new("tx15", vec!["Headphones".to_string()], Utc::now()),
    ];

    println!(
        "Dataset: {} transactions across multiple categories",
        transactions.len()
    );
    println!();

    let config = MiningConfig {
        min_support: 0.2,    // 20% - lower threshold for diverse data
        min_confidence: 0.7, // 70%
        min_lift: 1.3,       // Require strong correlation
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;
    let rules = miner.mine_association_rules()?;

    println!("Discovered {} high-quality rules:", rules.len());
    for (idx, rule) in rules.iter().enumerate() {
        println!(
            "{}. {:?} => {:?}",
            idx + 1,
            rule.antecedent,
            rule.consequent
        );
        println!(
            "   Confidence: {:.1}% | Support: {:.1}% | Lift: {:.2}",
            rule.metrics.confidence * 100.0,
            rule.metrics.support * 100.0,
            rule.metrics.lift
        );
    }
    println!();

    // Generate and save
    let grl = GrlExporter::to_grl(&rules);
    fs::write("/tmp/medium_rules.grl", &grl)?;
    println!("✓ Generated {} lines of GRL code", grl.lines().count());

    // Test multiple scenarios
    let mut engine = IncrementalEngine::new();
    GrlReteLoader::load_from_file("/tmp/medium_rules.grl", &mut engine)?;

    // Scenario 1: Gaming customer
    println!("\nTest Scenario 1: Customer buying Gaming Console");
    test_recommendation(&mut engine, vec!["Gaming Console"]);

    // Scenario 2: Phone customer
    println!("\nTest Scenario 2: Customer buying Phone");
    test_recommendation(&mut engine, vec!["Phone"]);

    // Scenario 3: Laptop customer
    println!("\nTest Scenario 3: Customer buying Laptop");
    test_recommendation(&mut engine, vec!["Laptop"]);

    Ok(())
}

fn test_recommendation(_engine: &mut IncrementalEngine, items: Vec<&str>) {
    // Create fresh engine instance
    let mut test_engine = IncrementalEngine::new();
    GrlReteLoader::load_from_file("/tmp/medium_rules.grl", &mut test_engine).unwrap();

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
    println!("  Rules fired: {}", fired.len());

    let wm = test_engine.working_memory();
    let recs = wm.get_by_type("Recommendation");
    if !recs.is_empty() {
        if let Some(FactValue::Array(rec_items)) = recs[0].data.get("items") {
            print!("  Recommended: [");
            for (i, item) in rec_items.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                if let FactValue::String(s) = item {
                    print!("\"{}\"", s);
                }
            }
            println!("]");
        }
    } else {
        println!("  Recommended: (none)");
    }
}
