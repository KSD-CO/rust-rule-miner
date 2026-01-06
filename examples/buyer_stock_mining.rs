//! Example: Mining Buyer Stock Data
//!
//! Demonstrates mining patterns from real buyer stock CSV data.
//! Shows different mining strategies:
//! 1. Product co-occurrence patterns (products updated together)
//! 2. Location-based product patterns
//! 3. SKU patterns
//!
//! Run with:
//! ```bash
//! cargo run --example buyer_stock_mining
//! ```

use rust_rule_miner::{
    data_loader::{ColumnMapping, DataLoader},
    export::GrlExporter,
    MiningAlgorithm, MiningConfig, RuleMiner,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Buyer Stock Data Mining ===\n");

    let csv_path = "examples/buyer_stock.csv";

    // Demo 1: Mine product patterns by location
    println!("--- Demo 1: Product Patterns by Location ---");
    mine_by_location(csv_path)?;
    println!();

    // Demo 2: Mine SKU patterns by location
    println!("--- Demo 2: SKU Patterns by Location ---");
    mine_sku_by_location(csv_path)?;
    println!();

    // Demo 3: Mine product categories (extract from product names)
    println!("--- Demo 3: Product Category Patterns ---");
    mine_product_categories(csv_path)?;
    println!();

    // Demo 4: Mine combined patterns (location + product + SKU)
    println!("--- Demo 4: Combined Multi-Field Patterns ---");
    mine_combined_patterns(csv_path)?;
    println!();

    // Demo 5: Aggregate by time window to get multi-item patterns
    println!("--- Demo 5: Time-Windowed Multi-Item Patterns ---");
    mine_time_windowed_patterns(csv_path)?;

    Ok(())
}

/// Mine product patterns grouped by location_id
fn mine_by_location(csv_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mining: Which products appear together at the same location?");
    println!();

    // CSV columns: id, location_id, sku, product_name, is_deleted, quantity, ...
    //              0   1            2    3             4           5
    // We'll use:
    // - location_id (column 1) as transaction_id (grouping)
    // - product_name (column 3) as items to mine
    // - updated_date (column 14) as timestamp

    let mapping = ColumnMapping::simple(1, 3, 14);
    println!("Column mapping: location_id(1) -> product_name(3) -> updated_date(14)");

    let transactions = DataLoader::from_csv(csv_path, mapping)?;
    println!(
        "✓ Loaded {} transactions (grouped by location)",
        transactions.len()
    );

    // Show sample
    if let Some(tx) = transactions.first() {
        println!("\nSample transaction:");
        println!("  Location ID: {}", tx.id);
        println!("  Products: {} items", tx.items.len());
        println!(
            "  First 3 products: {:?}",
            &tx.items.iter().take(3).collect::<Vec<_>>()
        );
    }

    // Mine with low support (locations may have different products)
    let config = MiningConfig {
        min_support: 0.01,   // 1% - rare patterns ok
        min_confidence: 0.5, // 50% confidence
        min_lift: 1.2,       // 20% lift
        algorithm: MiningAlgorithm::Apriori,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;

    println!("\nMining rules...");
    let rules = miner.mine_association_rules()?;

    println!("✓ Found {} association rules", rules.len());

    // Show top 5 rules
    if !rules.is_empty() {
        println!("\nTop 5 Rules (Products that appear together):");
        for (i, rule) in rules.iter().take(5).enumerate() {
            println!("\n{}. IF Location has:", i + 1);
            for item in rule.antecedent.iter().take(2) {
                println!("      - {}", truncate_string(item, 60));
            }
            println!("   THEN Also has:");
            for item in rule.consequent.iter().take(2) {
                println!("      - {}", truncate_string(item, 60));
            }
            println!(
                "   Confidence: {:.1}% | Support: {:.1}% | Lift: {:.2}",
                rule.metrics.confidence * 100.0,
                rule.metrics.support * 100.0,
                rule.metrics.lift
            );
        }

        // Export to GRL
        let grl = GrlExporter::to_grl(&rules);
        fs::write("/tmp/buyer_stock_location_rules.grl", &grl)?;
        println!(
            "\n✓ Exported {} rules to /tmp/buyer_stock_location_rules.grl",
            rules.len()
        );
    }

    Ok(())
}

/// Mine SKU patterns by location
fn mine_sku_by_location(csv_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mining: Which SKUs appear together at locations?");
    println!();

    // Use SKU (column 2) instead of product_name
    let mapping = ColumnMapping::simple(1, 2, 14);
    println!("Column mapping: location_id(1) -> sku(2) -> updated_date(14)");

    let transactions = DataLoader::from_csv(csv_path, mapping)?;
    println!("✓ Loaded {} SKU transactions", transactions.len());

    let config = MiningConfig {
        min_support: 0.02,
        min_confidence: 0.6,
        min_lift: 1.5,
        algorithm: MiningAlgorithm::Apriori,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;

    let rules = miner.mine_association_rules()?;
    println!("✓ Found {} SKU association rules", rules.len());

    // Show statistics
    if !rules.is_empty() {
        let avg_confidence =
            rules.iter().map(|r| r.metrics.confidence).sum::<f64>() / rules.len() as f64;
        let avg_lift = rules.iter().map(|r| r.metrics.lift).sum::<f64>() / rules.len() as f64;

        println!("\nRule Statistics:");
        println!("  Average Confidence: {:.1}%", avg_confidence * 100.0);
        println!("  Average Lift: {:.2}", avg_lift);

        // Show top rule
        if let Some(top_rule) = rules.first() {
            println!("\nTop Rule:");
            println!(
                "  {} => {}",
                top_rule.antecedent.join(", "),
                top_rule.consequent.join(", ")
            );
            println!("  Quality Score: {:.3}", top_rule.quality_score());
        }
    }

    Ok(())
}

/// Mine product categories extracted from product names
fn mine_product_categories(csv_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mining: Product category patterns");
    println!("Note: This would work better with actual category column");
    println!();

    // For demo, we'll use product_name and extract first word as "category"
    // In production, you'd have a proper category column

    let mapping = ColumnMapping::simple(1, 3, 14);
    let mut transactions = DataLoader::from_csv(csv_path, mapping)?;

    // Extract first word from product name as simple categorization
    for tx in &mut transactions {
        tx.items = tx
            .items
            .iter()
            .map(|product| {
                // Get first word (simple category extraction)
                product
                    .split_whitespace()
                    .next()
                    .unwrap_or(product)
                    .to_string()
            })
            .collect();
    }

    println!(
        "✓ Loaded and categorized {} transactions",
        transactions.len()
    );

    if let Some(tx) = transactions.first() {
        println!(
            "\nSample categories: {:?}",
            &tx.items.iter().take(5).collect::<Vec<_>>()
        );
    }

    let config = MiningConfig {
        min_support: 0.05,
        min_confidence: 0.6,
        min_lift: 1.3,
        algorithm: MiningAlgorithm::Apriori,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;

    let rules = miner.mine_association_rules()?;
    println!("\n✓ Found {} category association rules", rules.len());

    // Show insights
    if !rules.is_empty() {
        println!("\nTop 3 Category Patterns:");
        for (i, rule) in rules.iter().take(3).enumerate() {
            println!("{}. If location has: {}", i + 1, rule.antecedent.join(", "));
            println!(
                "   Then also has: {} ({:.1}% confidence)",
                rule.consequent.join(", "),
                rule.metrics.confidence * 100.0
            );
        }
    }

    Ok(())
}

/// Mine combined multi-field patterns (location + SKU + product)
fn mine_combined_patterns(csv_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mining: Combined location::SKU::product patterns");
    println!();

    // Combine multiple fields: location_id + sku + product_name
    // Columns: location_id(1), sku(2), product_name(3)
    let mapping = ColumnMapping::multi_field(
        0,                // id as transaction_id
        vec![1, 2, 3],    // combine location_id + sku + product_name
        14,               // updated_date as timestamp
        "::".to_string(), // separator
    );

    println!("Column mapping: id(0) -> [location(1), sku(2), product(3)] -> updated_date(14)");

    let transactions = DataLoader::from_csv(csv_path, mapping)?;
    println!(
        "✓ Loaded {} combined-field transactions",
        transactions.len()
    );

    // Show sample
    if let Some(tx) = transactions.first() {
        println!("\nSample combined patterns:");
        for (i, item) in tx.items.iter().take(3).enumerate() {
            println!("  {}. {}", i + 1, item);
        }
    }

    let config = MiningConfig {
        min_support: 0.001,  // Very low - combined patterns are rare
        min_confidence: 0.8, // But require high confidence
        min_lift: 2.0,       // And strong correlation
        algorithm: MiningAlgorithm::Apriori,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;

    let rules = miner.mine_association_rules()?;
    println!("\n✓ Found {} combined-pattern rules", rules.len());

    if !rules.is_empty() {
        println!("\nTop 3 Multi-Field Rules:");
        for (i, rule) in rules.iter().take(3).enumerate() {
            println!("\n{}. IF:", i + 1);
            for item in rule.antecedent.iter().take(2) {
                println!("      {}", truncate_string(item, 70));
            }
            println!("   THEN:");
            for item in rule.consequent.iter().take(2) {
                println!("      {}", truncate_string(item, 70));
            }
            println!(
                "   Confidence: {:.1}% | Lift: {:.2}",
                rule.metrics.confidence * 100.0,
                rule.metrics.lift
            );
        }

        // Export to GRL
        let grl = GrlExporter::to_grl(&rules);
        fs::write("/tmp/buyer_stock_combined_rules.grl", &grl)?;
        println!(
            "\n✓ Exported {} rules to /tmp/buyer_stock_combined_rules.grl",
            rules.len()
        );

        // Summary statistics
        let avg_confidence =
            rules.iter().map(|r| r.metrics.confidence).sum::<f64>() / rules.len() as f64;
        let avg_lift = rules.iter().map(|r| r.metrics.lift).sum::<f64>() / rules.len() as f64;
        println!("\nRule Quality Statistics:");
        println!("  Total Rules: {}", rules.len());
        println!("  Avg Confidence: {:.1}%", avg_confidence * 100.0);
        println!("  Avg Lift: {:.2}", avg_lift);
    } else {
        println!("\nNo rules found with current thresholds.");
        println!("Try adjusting min_support, min_confidence, or min_lift.");
    }

    Ok(())
}

/// Mine patterns by aggregating products within time windows
/// This creates multi-item transactions and discovers complex patterns
fn mine_time_windowed_patterns(csv_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Mining: Multi-item patterns using time-window aggregation");
    println!("Strategy: Group products by location + 1-hour time windows");
    println!();

    use std::collections::HashMap;

    // Load all data first with standard mapping
    let mapping = ColumnMapping::simple(1, 3, 14);
    let raw_transactions = DataLoader::from_csv(csv_path, mapping)?;
    println!("✓ Loaded {} raw product updates", raw_transactions.len());

    // Aggregate by location + hour
    let mut aggregated: HashMap<String, Vec<String>> = HashMap::new();

    for tx in raw_transactions {
        // Create key: location + hour (rounded down)
        let hour = tx.timestamp.format("%Y-%m-%d %H:00").to_string();
        let key = format!("{}_{}", tx.id, hour);

        // Add all items to this aggregated transaction
        aggregated.entry(key).or_default().extend(tx.items);
    }

    // Convert back to Transaction objects
    use chrono::Utc;
    let aggregated_transactions: Vec<rust_rule_miner::Transaction> = aggregated
        .into_iter()
        .enumerate()
        .map(|(idx, (_key, items))| {
            rust_rule_miner::Transaction::new(format!("agg_{}", idx), items, Utc::now())
        })
        .filter(|tx| tx.items.len() > 1) // Only keep transactions with 2+ items
        .collect();

    println!(
        "✓ Aggregated into {} multi-item transactions",
        aggregated_transactions.len()
    );

    // Show sample
    if let Some(tx) = aggregated_transactions.first() {
        println!("\nSample aggregated transaction:");
        println!("  ID: {}", tx.id);
        println!("  Items: {} products", tx.items.len());
        println!(
            "  First 5: {:?}",
            &tx.items.iter().take(5).collect::<Vec<_>>()
        );
    }

    // Mine with lower support to find patterns
    let config = MiningConfig {
        min_support: 0.005,  // 0.5% - very low for rare patterns
        min_confidence: 0.6, // 60% confidence
        min_lift: 2.0,       // Strong correlation
        algorithm: MiningAlgorithm::Apriori,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(aggregated_transactions)?;

    println!("\nMining association rules...");
    let rules = miner.mine_association_rules()?;

    println!("✓ Found {} multi-item association rules", rules.len());

    if !rules.is_empty() {
        // Sort by antecedent size (prefer multi-item patterns)
        let mut sorted_rules = rules.clone();
        sorted_rules.sort_by_key(|r| std::cmp::Reverse(r.antecedent.len() + r.consequent.len()));

        println!("\nTop 10 Complex Patterns (multi-item):");
        for (i, rule) in sorted_rules.iter().take(10).enumerate() {
            let total_items = rule.antecedent.len() + rule.consequent.len();
            println!("\n{}. Pattern Size: {} items total", i + 1, total_items);

            println!("   IF Customer bought:");
            for item in rule.antecedent.iter().take(3) {
                println!("      • {}", truncate_string(item, 60));
            }
            if rule.antecedent.len() > 3 {
                println!("      ... and {} more", rule.antecedent.len() - 3);
            }

            println!("   THEN Also bought:");
            for item in rule.consequent.iter().take(3) {
                println!("      • {}", truncate_string(item, 60));
            }
            if rule.consequent.len() > 3 {
                println!("      ... and {} more", rule.consequent.len() - 3);
            }

            println!(
                "   Metrics: Conf={:.1}% | Sup={:.1}% | Lift={:.2}",
                rule.metrics.confidence * 100.0,
                rule.metrics.support * 100.0,
                rule.metrics.lift
            );
        }

        // Export to GRL
        let grl = GrlExporter::to_grl(&rules);
        fs::write("/tmp/buyer_stock_time_windowed_rules.grl", &grl)?;
        println!(
            "\n✓ Exported {} rules to /tmp/buyer_stock_time_windowed_rules.grl",
            rules.len()
        );

        // Statistics
        let multi_item_rules: Vec<_> = rules
            .iter()
            .filter(|r| r.antecedent.len() > 1 || r.consequent.len() > 1)
            .collect();

        println!("\nPattern Complexity Statistics:");
        println!("  Total Rules: {}", rules.len());
        println!(
            "  Multi-item patterns: {} ({:.1}%)",
            multi_item_rules.len(),
            multi_item_rules.len() as f64 / rules.len() as f64 * 100.0
        );

        if let Some(max_rule) = sorted_rules.first() {
            let max_size = max_rule.antecedent.len() + max_rule.consequent.len();
            println!("  Largest pattern: {} items", max_size);
        }
    } else {
        println!("\nNo rules found with current thresholds.");
        println!("Suggestions:");
        println!("  - Lower min_support (currently 0.5%)");
        println!("  - Lower min_confidence (currently 60%)");
        println!("  - Use longer time windows (currently 1 hour)");
    }

    Ok(())
}

/// Helper function to truncate long strings for display
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
