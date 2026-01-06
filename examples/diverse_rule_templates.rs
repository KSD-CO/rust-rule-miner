//! Example: Diverse Rule Templates - ML-like Rich Rule Patterns
//!
//! Demonstrates 8 different rule templates for various use cases:
//! - Recommendation (e-commerce)
//! - Alert (security/monitoring)
//! - Classification (ML-like categorization)
//! - Scoring (risk/quality scoring)
//! - Validation (data quality)
//! - MultiAction (complex workflows)
//! - FraudDetection (anomaly detection)
//! - InventoryAlert (supply chain)
//!
//! Run with:
//! ```bash
//! cargo run --example diverse_rule_templates
//! ```

use rust_rule_miner::{
    data_loader::{ColumnMapping, DataLoader},
    export::GrlExporter,
    GrlConfig, MiningAlgorithm, MiningConfig, RuleMiner, RuleTemplate,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Diverse Rule Template Showcase ===\n");
    println!("Generating ML-like rich rule patterns from buyer stock data\n");

    let csv_path = "examples/buyer_stock.csv";

    // Load and aggregate data (same as multi_item example)
    let mapping = ColumnMapping::simple(1, 3, 14);
    let raw_transactions = DataLoader::from_csv(csv_path, mapping)?;
    println!("✓ Loaded {} product updates", raw_transactions.len());

    // Aggregate by location + hour window
    use chrono::Utc;
    use std::collections::HashMap;

    let mut aggregated: HashMap<String, Vec<String>> = HashMap::new();
    for tx in raw_transactions {
        let hour = tx.timestamp.format("%Y-%m-%d %H:00").to_string();
        let key = format!("loc{}_{}h", tx.id, hour);
        aggregated.entry(key).or_default().extend(tx.items);
    }

    let mut multi_item_txs: Vec<rust_rule_miner::Transaction> = aggregated
        .into_iter()
        .enumerate()
        .map(|(idx, (_key, items))| {
            rust_rule_miner::Transaction::new(format!("agg_{}", idx), items, Utc::now())
        })
        .filter(|tx| tx.items.len() >= 2)
        .collect();

    // Deduplicate items
    for tx in multi_item_txs.iter_mut() {
        let mut unique_items: Vec<String> = tx.items.clone();
        unique_items.sort();
        unique_items.dedup();
        tx.items = unique_items;
    }

    println!(
        "✓ Created {} multi-item transactions\n",
        multi_item_txs.len()
    );

    // Mine association rules once
    let config = MiningConfig {
        min_support: 0.2,
        min_confidence: 0.70,
        min_lift: 2.0,
        algorithm: MiningAlgorithm::FPGrowth,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(multi_item_txs)?;
    let rules = miner.mine_association_rules()?;

    println!("✓ Found {} association rules\n", rules.len());
    println!("Now exporting to 8 different rule template formats...\n");
    println!("{}\n", "=".repeat(80));

    // 1. RECOMMENDATION RULES (Default)
    println!("📊 1. RECOMMENDATION RULES - E-commerce Product Suggestions");
    println!("{}", "-".repeat(80));
    let config = GrlConfig::default();
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/rules_recommendation.grl", &grl)?;
    println!("✓ Generated recommendation rules → /tmp/rules_recommendation.grl");
    print_sample_rule(&grl);
    println!();

    // 2. ALERT RULES - Security/Monitoring
    println!("🚨 2. ALERT RULES - Security & Monitoring Patterns");
    println!("{}", "-".repeat(80));
    let config = GrlConfig::alert("Transaction.items");
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/rules_alert.grl", &grl)?;
    println!("✓ Generated alert rules → /tmp/rules_alert.grl");
    print_sample_rule(&grl);
    println!();

    // 3. CLASSIFICATION RULES - ML-like Categorization
    println!("🏷️  3. CLASSIFICATION RULES - ML-like Category Assignment");
    println!("{}", "-".repeat(80));
    let config = GrlConfig::classification("Order.items", "Order.category");
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/rules_classification.grl", &grl)?;
    println!("✓ Generated classification rules → /tmp/rules_classification.grl");
    print_sample_rule(&grl);
    println!();

    // 4. SCORING RULES - Risk/Quality Scoring
    println!("📈 4. SCORING RULES - Risk & Quality Scoring");
    println!("{}", "-".repeat(80));
    let config = GrlConfig::scoring("Purchase.items", "RiskScore.value");
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/rules_scoring.grl", &grl)?;
    println!("✓ Generated scoring rules → /tmp/rules_scoring.grl");
    print_sample_rule(&grl);
    println!();

    // 5. VALIDATION RULES - Data Quality Checks
    println!("✅ 5. VALIDATION RULES - Data Quality & Completeness");
    println!("{}", "-".repeat(80));
    let config = GrlConfig::default().with_template(RuleTemplate::Validation);
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/rules_validation.grl", &grl)?;
    println!("✓ Generated validation rules → /tmp/rules_validation.grl");
    print_sample_rule(&grl);
    println!();

    // 6. MULTI-ACTION RULES - Complex Workflows
    println!("⚙️  6. MULTI-ACTION RULES - Complex Workflow Orchestration");
    println!("{}", "-".repeat(80));
    let config = GrlConfig::default()
        .with_template(RuleTemplate::MultiAction)
        .with_action_prefix("Workflow");
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/rules_multiaction.grl", &grl)?;
    println!("✓ Generated multi-action rules → /tmp/rules_multiaction.grl");
    print_sample_rule(&grl);
    println!();

    // 7. FRAUD DETECTION RULES - Anomaly Detection
    println!("🔍 7. FRAUD DETECTION RULES - Anomaly & Pattern Detection");
    println!("{}", "-".repeat(80));
    let config = GrlConfig::fraud_detection("Transaction.items");
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/rules_fraud.grl", &grl)?;
    println!("✓ Generated fraud detection rules → /tmp/rules_fraud.grl");
    print_sample_rule(&grl);
    println!();

    // 8. INVENTORY ALERT RULES - Supply Chain Management
    println!("📦 8. INVENTORY ALERT RULES - Supply Chain & Stock Management");
    println!("{}", "-".repeat(80));
    let config = GrlConfig::inventory_alert("Stock.items");
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/rules_inventory.grl", &grl)?;
    println!("✓ Generated inventory alert rules → /tmp/rules_inventory.grl");
    print_sample_rule(&grl);
    println!();

    // Summary
    println!("{}", "=".repeat(80));
    println!("\n✨ SUMMARY - 8 Rule Template Types Generated\n");
    println!("📊 Recommendation  → Product suggestions & cross-sell");
    println!("🚨 Alert          → Security monitoring & anomaly detection");
    println!("🏷️  Classification → ML-like category assignment");
    println!("📈 Scoring        → Risk/quality scoring systems");
    println!("✅ Validation     → Data quality & completeness checks");
    println!("⚙️  MultiAction    → Complex workflow orchestration");
    println!("🔍 Fraud Detection → Transaction anomaly patterns");
    println!("📦 Inventory Alert → Supply chain management");
    println!();
    println!("All rules exported to /tmp/rules_*.grl");
    println!("\n🎯 Key Insight: Same patterns → 8 different business applications!");
    println!("This is how you achieve ML-like richness with association rules.\n");

    Ok(())
}

fn print_sample_rule(grl: &str) {
    // Extract first rule's "when...then" block
    if let Some(start) = grl.find("rule \"") {
        if let Some(when_pos) = grl[start..].find("when") {
            if let Some(end) = grl[start + when_pos..].find("}}") {
                let sample = &grl[start + when_pos..start + when_pos + end + 2];
                let lines: Vec<&str> = sample.lines().take(10).collect();
                println!("Sample:");
                for line in lines {
                    println!("  {}", line);
                }
                if sample.lines().count() > 10 {
                    println!("  ...");
                }
            }
        }
    }
}
