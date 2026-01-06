//! Example: SKU Reorder Prediction - Predict which SKUs to reorder based on sales patterns
//!
//! Use case: Inventory Management & Purchase Planning
//! - Analyze which SKUs are frequently sold together
//! - Predict reorder needs based on current sales patterns
//! - Generate purchase order recommendations
//!
//! Business logic:
//! IF SKU_A is selling → THEN reorder SKU_B (they often sell together)
//!
//! Run with:
//! ```bash
//! cargo run --example sku_reorder_prediction
//! ```

use chrono::Timelike;
use rust_rule_miner::{
    data_loader::{ColumnMapping, DataLoader},
    export::GrlExporter,
    GrlConfig, MiningAlgorithm, MiningConfig, RuleMiner, RuleTemplate,
};
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SKU Reorder Prediction System ===\n");
    println!("📦 Phân tích patterns để dự đoán SKU cần đặt hàng\n");

    let csv_path = "examples/buyer_stock.csv";

    // Load data with SKU column (column 2)
    println!("📊 Step 1: Load dữ liệu SKU từ lịch sử bán hàng");
    let mapping = ColumnMapping::simple(1, 2, 14); // location_id, SKU, updated_date
    let raw_transactions = DataLoader::from_csv(csv_path, mapping)?;
    println!("✓ Loaded {} SKU update records", raw_transactions.len());

    // Filter out empty SKUs
    let transactions: Vec<_> = raw_transactions
        .into_iter()
        .filter(|tx| !tx.items.is_empty() && !tx.items[0].trim().is_empty())
        .collect();
    println!("✓ Filtered to {} valid SKU records\n", transactions.len());

    // Aggregate by time windows (4-hour blocks) to see which SKUs sold together
    // Use 4-hour windows to reduce transaction size for faster mining
    println!("📊 Step 2: Aggregate SKUs theo khung giờ 4h");
    let mut time_window_sales: HashMap<String, Vec<String>> = HashMap::new();

    for tx in transactions {
        // Group by 4-hour blocks (00-04, 04-08, 08-12, 12-16, 16-20, 20-24)
        let hour = tx.timestamp.hour();
        let block = (hour / 4) * 4;
        let time_key = tx
            .timestamp
            .format(&format!("%Y-%m-%d {:02}:00", block))
            .to_string();

        for sku in tx.items {
            let sku_trimmed = sku.trim().to_string();
            if !sku_trimmed.is_empty() {
                time_window_sales
                    .entry(time_key.clone())
                    .or_default()
                    .push(sku_trimmed);
            }
        }
    }

    // Convert to transactions (each 4-hour window = 1 transaction)
    use chrono::Utc;
    let mut sku_transactions: Vec<rust_rule_miner::Transaction> = time_window_sales
        .into_iter()
        .map(|(time_window, skus)| {
            rust_rule_miner::Transaction::new(format!("window_{}", time_window), skus, Utc::now())
        })
        .collect();

    // Deduplicate SKUs within each window
    for tx in sku_transactions.iter_mut() {
        let mut unique_skus: Vec<String> = tx.items.clone();
        unique_skus.sort();
        unique_skus.dedup();
        tx.items = unique_skus;
    }

    // Filter to only windows with 2+ SKUs and limit to max 50 SKUs per transaction
    sku_transactions.retain(|tx| tx.items.len() >= 2 && tx.items.len() <= 50);

    println!(
        "✓ Created {} time-window transactions",
        sku_transactions.len()
    );

    // Show sample
    if let Some(sample) = sku_transactions.first() {
        println!("\nSample transaction:");
        println!("  Date: {}", sample.id);
        println!("  SKUs sold: {}", sample.items.len());
        for (i, sku) in sample.items.iter().take(5).enumerate() {
            println!("    {}. {}", i + 1, sku);
        }
        if sample.items.len() > 5 {
            println!("    ... and {} more", sample.items.len() - 5);
        }
    }
    println!();

    // Mine association rules
    println!("📊 Step 3: Mine SKU association patterns");
    println!("Tìm patterns: SKU nào bán → SKU nào cần đặt hàng\n");

    let config = MiningConfig {
        min_support: 0.15,    // 15% - SKU patterns phải xuất hiện >= 15% ngày
        min_confidence: 0.65, // 65% - độ tin cậy cao
        min_lift: 2.0,        // 2.0 - tương quan mạnh
        algorithm: MiningAlgorithm::FPGrowth,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(sku_transactions)?;
    let rules = miner.mine_association_rules()?;

    println!("✓ Found {} SKU reorder patterns\n", rules.len());

    if rules.is_empty() {
        println!("⚠️  No patterns found with current thresholds!");
        println!("Suggestions:");
        println!("  - Lower min_support (currently 15%)");
        println!("  - Lower min_confidence (currently 65%)");
        println!("  - Lower min_lift (currently 2.0)");
        return Ok(());
    }

    // Display top reorder recommendations
    let separator = "=".repeat(80);
    println!("{}", separator);
    println!("🎯 TOP SKU REORDER RECOMMENDATIONS");
    println!("{}", separator);
    println!();

    // Sort by confidence * lift (quality score)
    let mut sorted_rules = rules.clone();
    sorted_rules.sort_by(|a, b| {
        let score_a = a.metrics.confidence * a.metrics.lift;
        let score_b = b.metrics.confidence * b.metrics.lift;
        score_b.partial_cmp(&score_a).unwrap()
    });

    for (i, rule) in sorted_rules.iter().take(15).enumerate() {
        println!("{}. REORDER RECOMMENDATION", i + 1);
        println!("   {}", "-".repeat(70));

        println!("   📍 WHEN these SKUs are selling:");
        for sku in rule.antecedent.iter().take(3) {
            println!("      ✓ {}", truncate(sku, 60));
        }
        if rule.antecedent.len() > 3 {
            println!("      ... and {} more SKUs", rule.antecedent.len() - 3);
        }

        println!("\n   📦 THEN reorder these SKUs:");
        for sku in rule.consequent.iter().take(3) {
            println!("      → {}", truncate(sku, 60));
        }
        if rule.consequent.len() > 3 {
            println!("      ... and {} more SKUs", rule.consequent.len() - 3);
        }

        // Calculate priority score
        let priority_score = (rule.metrics.confidence * rule.metrics.lift * 100.0) as i32;
        let priority = if priority_score > 400 {
            "🔴 HIGH"
        } else if priority_score > 250 {
            "🟡 MEDIUM"
        } else {
            "🟢 LOW"
        };

        println!("\n   📊 Metrics:");
        println!(
            "      Confidence: {:.1}% (SKUs xuất hiện cùng nhau {:.1}% thời gian)",
            rule.metrics.confidence * 100.0,
            rule.metrics.confidence * 100.0
        );
        println!(
            "      Support: {:.1}% (Pattern xuất hiện trong {:.1}% ngày)",
            rule.metrics.support * 100.0,
            rule.metrics.support * 100.0
        );
        println!(
            "      Lift: {:.2}x (Tương quan mạnh gấp {:.2} lần)",
            rule.metrics.lift, rule.metrics.lift
        );
        println!("      Priority: {} (Score: {})", priority, priority_score);
        println!();
    }

    // Export to different formats
    println!("{}", separator);
    println!("📤 EXPORTING REORDER RULES");
    println!("{}", separator);
    println!();

    // 1. Inventory Alert format
    println!("1. Inventory Alert Rules (for automation systems)");
    let config = GrlConfig::inventory_alert("CurrentSales.skus");
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/sku_reorder_inventory_alert.grl", &grl)?;
    println!(
        "   ✓ /tmp/sku_reorder_inventory_alert.grl ({} KB)",
        grl.len() / 1024
    );

    // 2. Recommendation format (for purchase managers)
    println!("2. Purchase Recommendation Rules");
    let config = GrlConfig::default().with_template(RuleTemplate::Recommendation);
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/sku_reorder_recommendations.grl", &grl)?;
    println!(
        "   ✓ /tmp/sku_reorder_recommendations.grl ({} KB)",
        grl.len() / 1024
    );

    // 3. Scoring format (for priority ranking)
    println!("3. Priority Scoring Rules");
    let config = GrlConfig::scoring("Sales.skus", "ReorderPriority.score");
    let grl = GrlExporter::to_grl_with_config(&rules, &config);
    fs::write("/tmp/sku_reorder_priority_scoring.grl", &grl)?;
    println!(
        "   ✓ /tmp/sku_reorder_priority_scoring.grl ({} KB)",
        grl.len() / 1024
    );

    // Generate CSV summary for purchase managers
    println!("\n4. CSV Summary (for Excel/Spreadsheet)");
    let csv_content = generate_csv_summary(&sorted_rules);
    fs::write("/tmp/sku_reorder_summary.csv", csv_content)?;
    println!("   ✓ /tmp/sku_reorder_summary.csv");

    // Statistics
    println!();
    println!("{}", separator);
    println!("📈 SUMMARY STATISTICS");
    println!("{}", separator);

    let avg_confidence =
        rules.iter().map(|r| r.metrics.confidence).sum::<f64>() / rules.len() as f64;
    let avg_lift = rules.iter().map(|r| r.metrics.lift).sum::<f64>() / rules.len() as f64;
    let high_priority = rules
        .iter()
        .filter(|r| r.metrics.confidence * r.metrics.lift > 4.0)
        .count();

    println!("\nTotal Reorder Rules: {}", rules.len());
    println!(
        "High Priority Rules: {} ({:.1}%)",
        high_priority,
        high_priority as f64 / rules.len() as f64 * 100.0
    );
    println!("Average Confidence: {:.1}%", avg_confidence * 100.0);
    println!("Average Lift: {:.2}x", avg_lift);

    // Unique SKUs involved
    let mut unique_trigger_skus = std::collections::HashSet::new();
    let mut unique_reorder_skus = std::collections::HashSet::new();
    for rule in &rules {
        for sku in &rule.antecedent {
            unique_trigger_skus.insert(sku.clone());
        }
        for sku in &rule.consequent {
            unique_reorder_skus.insert(sku.clone());
        }
    }

    println!("\nUnique Trigger SKUs: {}", unique_trigger_skus.len());
    println!("Unique Reorder SKUs: {}", unique_reorder_skus.len());

    println!("\n✅ Done! Use these rules to automate reorder decisions.");
    println!("\n💡 Next steps:");
    println!("   1. Review CSV summary: /tmp/sku_reorder_summary.csv");
    println!("   2. Integrate GRL rules into inventory system");
    println!("   3. Set up alerts for high-priority reorder patterns");
    println!("   4. Monitor rule performance over time");

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        s.chars().take(max_len).collect::<String>() + "..."
    }
}

fn generate_csv_summary(rules: &[rust_rule_miner::AssociationRule]) -> String {
    let mut csv =
        String::from("Priority,Trigger_SKUs,Reorder_SKUs,Confidence_%,Support_%,Lift,Score\n");

    for rule in rules.iter().take(50) {
        let trigger_skus = rule.antecedent.join("; ");
        let reorder_skus = rule.consequent.join("; ");
        let score = (rule.metrics.confidence * rule.metrics.lift * 100.0) as i32;
        let priority = if score > 400 {
            "HIGH"
        } else if score > 250 {
            "MEDIUM"
        } else {
            "LOW"
        };

        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",{:.1},{:.1},{:.2},{}\n",
            priority,
            trigger_skus,
            reorder_skus,
            rule.metrics.confidence * 100.0,
            rule.metrics.support * 100.0,
            rule.metrics.lift,
            score
        ));
    }

    csv
}
