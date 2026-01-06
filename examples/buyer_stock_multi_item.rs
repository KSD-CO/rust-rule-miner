//! Example: Multi-Item Pattern Mining from Buyer Stock
//!
//! Demonstrates aggregating data by time windows to discover
//! complex multi-item association rules.
//!
//! Run with:
//! ```bash
//! cargo run --example buyer_stock_multi_item
//! ```

use rust_rule_miner::{
    data_loader::{ColumnMapping, DataLoader},
    export::GrlExporter,
    MiningAlgorithm, MiningConfig, RuleMiner,
};
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Item Pattern Mining ===\n");

    let csv_path = "examples/buyer_stock.csv";

    // Load all product updates
    let mapping = ColumnMapping::simple(1, 3, 14); // location_id, product_name, updated_date
    let raw_transactions = DataLoader::from_csv(csv_path, mapping)?;
    println!("✓ Loaded {} product updates", raw_transactions.len());

    // Aggregate by location + hour window
    println!("\nAggregating by location + 1-hour time windows...");
    let mut aggregated: HashMap<String, Vec<String>> = HashMap::new();

    for tx in raw_transactions {
        // Create key: location_id + hour
        let hour = tx.timestamp.format("%Y-%m-%d %H:00").to_string();
        let key = format!("loc{}_{}h", tx.id, hour);

        // Collect all products in this window
        aggregated.entry(key).or_default().extend(tx.items);
    }

    // Convert to multi-item transactions
    use chrono::Utc;
    let mut multi_item_txs: Vec<rust_rule_miner::Transaction> = aggregated
        .into_iter()
        .enumerate()
        .map(|(idx, (_key, items))| {
            rust_rule_miner::Transaction::new(format!("agg_{}", idx), items, Utc::now())
        })
        .filter(|tx| tx.items.len() >= 2) // Only keep transactions with 2+ products
        .collect();

    println!("✓ Created {} multi-item transactions", multi_item_txs.len());

    // Show samples
    println!("\nSample Aggregated Transactions:");
    for (i, tx) in multi_item_txs.iter().take(3).enumerate() {
        println!("{}. ID: {} | Products: {}", i + 1, tx.id, tx.items.len());
        for (j, item) in tx.items.iter().take(5).enumerate() {
            println!("   {}. {}", j + 1, truncate(item, 60));
        }
        if tx.items.len() > 5 {
            println!("   ... and {} more", tx.items.len() - 5);
        }
    }

    // Deduplicate items in each transaction
    for tx in multi_item_txs.iter_mut() {
        let mut unique_items: Vec<String> = tx.items.clone();
        unique_items.sort();
        unique_items.dedup();
        tx.items = unique_items;
    }

    println!("Deduplicated items in transactions");

    // Use all transactions for comprehensive mining
    println!("Using all {} transactions for mining", multi_item_txs.len());

    // Mine with very high-quality thresholds for best patterns
    println!("\nMining association rules with very high-quality thresholds...");
    let config = MiningConfig {
        min_support: 0.2,     // 20% - only very frequent patterns
        min_confidence: 0.70, // 70% - high confidence rules
        min_lift: 2.0,        // 2.0 - strong correlations only
        algorithm: MiningAlgorithm::FPGrowth,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(multi_item_txs)?;
    let rules = miner.mine_association_rules()?;

    println!("✓ Found {} association rules", rules.len());

    if rules.is_empty() {
        println!("\n⚠️  No rules found!");
        println!("Try:");
        println!("  - Lower min_support (currently 20%)");
        println!("  - Lower min_confidence (currently 70%)");
        println!("  - Lower min_lift (currently 2.0)");
        println!("  - Use longer time windows or more transactions");
        return Ok(());
    }

    // Analyze and display
    let multi_item_rules: Vec<_> = rules
        .iter()
        .filter(|r| r.antecedent.len() > 1 || r.consequent.len() > 1)
        .collect();

    println!("\n📊 Rule Statistics:");
    println!("  Total rules: {}", rules.len());
    println!(
        "  Multi-item patterns: {} ({:.1}%)",
        multi_item_rules.len(),
        multi_item_rules.len() as f64 / rules.len() as f64 * 100.0
    );

    // Show top complex patterns
    let mut sorted = rules.clone();
    sorted.sort_by_key(|r| std::cmp::Reverse(r.antecedent.len() + r.consequent.len()));

    println!("\n🔍 Top 10 Complex Patterns:");
    for (i, rule) in sorted.iter().take(10).enumerate() {
        let size = rule.antecedent.len() + rule.consequent.len();
        println!(
            "\n{}. Pattern: {} → {} ({} items total)",
            i + 1,
            rule.antecedent.len(),
            rule.consequent.len(),
            size
        );

        println!("   IF:");
        for item in rule.antecedent.iter().take(4) {
            println!("      ✓ {}", truncate(item, 60));
        }
        if rule.antecedent.len() > 4 {
            println!("      ... +{} more", rule.antecedent.len() - 4);
        }

        println!("   THEN:");
        for item in rule.consequent.iter().take(4) {
            println!("      → {}", truncate(item, 60));
        }
        if rule.consequent.len() > 4 {
            println!("      ... +{} more", rule.consequent.len() - 4);
        }

        println!(
            "   Confidence: {:.1}% | Support: {:.1}% | Lift: {:.2}",
            rule.metrics.confidence * 100.0,
            rule.metrics.support * 100.0,
            rule.metrics.lift
        );
    }

    // Export with default template (Recommendation)
    let grl = GrlExporter::to_grl(&rules);
    let output_path = "/tmp/buyer_stock_multi_item_rules.grl";
    fs::write(output_path, &grl)?;
    println!("\n✅ Exported {} rules to {}", rules.len(), output_path);
    println!("   File size: {} KB", grl.len() / 1024);

    // Show sample GRL for multi-item rule
    if let Some(complex_rule) = sorted.first() {
        if complex_rule.antecedent.len() > 1 {
            println!("\n📝 Sample Multi-Item GRL Rule:");
            println!("```grl");
            println!("when");
            for (i, item) in complex_rule.antecedent.iter().enumerate() {
                let connector = if i == 0 { "" } else { " &&\n    " };
                println!("    {}ShoppingCart.items contains \"{}\"", connector, item);
            }
            println!("then");
            for item in complex_rule.consequent.iter() {
                println!("    Recommendation.items += \"{}\";", item);
            }
            println!("```");
        }
    }

    // 🎯 BONUS: Export with different rule templates for diverse applications
    println!("\n💡 Tip: You can export the same rules with different templates!");
    println!("See examples/diverse_rule_templates.rs for 8 different rule types:");
    println!("  • Recommendation (default) - E-commerce suggestions");
    println!("  • Alert - Security monitoring");
    println!("  • Classification - ML-like categorization");
    println!("  • Scoring - Risk/quality scoring");
    println!("  • FraudDetection - Anomaly detection");
    println!("  • InventoryAlert - Supply chain management");
    println!("  • Validation - Data quality checks");
    println!("  • MultiAction - Complex workflows");
    println!("\nRun: cargo run --example diverse_rule_templates");

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        s.chars().take(max_len).collect::<String>() + "..."
    }
}
