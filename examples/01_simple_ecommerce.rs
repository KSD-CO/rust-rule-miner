//! Example 1: Simple E-commerce Recommendations
//!
//! Demonstrates basic rule mining from small dataset.
//! Use case: Product recommendations based on purchase history.
//!
//! NOTE: For custom field names beyond ShoppingCart.items, use GrlConfig:
//!   use rust_rule_miner::export::GrlConfig;
//!   let config = GrlConfig::custom("Order.items", "Suggestions.products");
//!   let grl = GrlExporter::to_grl_with_config(&rules, &config);
//!
//! See examples/flexible_domain_mining.rs for more domains.

use chrono::Utc;
use rust_rule_engine::rete::{
    facts::{FactValue, TypedFacts},
    grl_loader::GrlReteLoader,
    propagation::IncrementalEngine,
};
use rust_rule_miner::{export::GrlExporter, MiningConfig, RuleMiner, Transaction};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 1: Simple E-commerce Recommendations ===\n");

    // Small dataset: 6 transactions
    let transactions = vec![
        Transaction::new(
            "tx1",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx2",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx3",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx4",
            vec!["Phone".to_string(), "Phone Case".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx5",
            vec!["Phone".to_string(), "Phone Case".to_string()],
            Utc::now(),
        ),
        Transaction::new("tx6", vec!["Tablet".to_string()], Utc::now()),
    ];

    println!("Dataset: {} transactions", transactions.len());
    println!();

    // Simple config
    let config = MiningConfig {
        min_support: 0.3,    // 30%
        min_confidence: 0.8, // 80%
        min_lift: 1.0,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;
    let rules = miner.mine_association_rules()?;

    println!("Discovered {} rules:", rules.len());
    for rule in &rules {
        println!(
            "  {:?} => {:?} (conf: {:.0}%, support: {:.0}%, lift: {:.2})",
            rule.antecedent,
            rule.consequent,
            rule.metrics.confidence * 100.0,
            rule.metrics.support * 100.0,
            rule.metrics.lift
        );
    }
    println!();

    // Generate and test
    let grl = GrlExporter::to_grl(&rules);
    fs::write("/tmp/simple_rules.grl", &grl)?;

    let mut engine = IncrementalEngine::new();
    GrlReteLoader::load_from_file("/tmp/simple_rules.grl", &mut engine)?;

    // Test: Customer adds Laptop
    let mut cart = TypedFacts::new();
    cart.set(
        "items",
        FactValue::Array(vec![FactValue::String("Laptop".to_string())]),
    );

    let mut rec = TypedFacts::new();
    rec.set("items", FactValue::Array(vec![]));

    engine.insert("ShoppingCart".to_string(), cart);
    engine.insert("Recommendation".to_string(), rec);

    let fired = engine.fire_all();
    println!("Test: Customer adds Laptop to cart");
    println!("  Rules fired: {}", fired.len());

    let wm = engine.working_memory();
    let recs = wm.get_by_type("Recommendation");
    if !recs.is_empty() {
        if let Some(FactValue::Array(items)) = recs[0].data.get("items") {
            print!("  Recommendations: [");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                if let FactValue::String(s) = item {
                    print!("\"{}\"", s);
                }
            }
            println!("]");
        }
    }

    Ok(())
}
