//! Example 4: Loading Data from Excel and CSV Files
//!
//! Demonstrates loading transaction data from files using excelstream.
//! Use case: Import historical sales data for pattern mining.

use rust_rule_miner::{data_loader::DataLoader, MiningConfig, RuleMiner};
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 4: Loading Data from Excel/CSV ===\n");

    // Create sample CSV file
    create_sample_csv()?;

    // Demo 1: Load from CSV
    demo_csv_loading()?;

    // Demo 2: Mine rules from CSV data
    demo_csv_mining()?;

    // Cleanup
    cleanup_temp_files();

    Ok(())
}

fn create_sample_csv() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Creating Sample CSV File ---");

    let csv_content = r#"transaction_id,items,timestamp
tx001,"Laptop,Mouse,Keyboard",2024-01-01T10:00:00Z
tx002,"Laptop,Mouse",2024-01-01T11:00:00Z
tx003,"Laptop,Mouse,USB-C Hub",2024-01-01T12:00:00Z
tx004,"Phone,Phone Case",2024-01-02T09:00:00Z
tx005,"Phone,Phone Case,Screen Protector",2024-01-02T10:00:00Z
tx006,"Phone,Phone Case,Charger",2024-01-02T11:00:00Z
tx007,"Tablet,Tablet Case",2024-01-03T08:00:00Z
tx008,"Tablet,Stylus",2024-01-03T09:00:00Z
tx009,"Camera,Memory Card,Camera Bag",2024-01-04T10:00:00Z
tx010,"Camera,Memory Card,Tripod",2024-01-04T11:00:00Z
tx011,"Gaming Console,Controller,Game",2024-01-05T13:00:00Z
tx012,"Gaming Console,Controller",2024-01-05T14:00:00Z
tx013,"Monitor,HDMI Cable",2024-01-06T10:00:00Z
tx014,"Monitor,HDMI Cable,Monitor Stand",2024-01-06T11:00:00Z
tx015,"Headphones,Audio Cable",2024-01-07T12:00:00Z
"#;

    let file_path = "/tmp/sample_transactions.csv";
    let mut file = fs::File::create(file_path)?;
    file.write_all(csv_content.as_bytes())?;

    println!("✓ Created: {}", file_path);
    println!("  15 transactions with various product categories");
    println!();

    Ok(())
}

fn demo_csv_loading() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 1: Loading from CSV ---");

    let file_path = "/tmp/sample_transactions.csv";

    // Load transactions using excelstream
    let transactions = DataLoader::from_csv(file_path)?;

    println!("✓ Loaded {} transactions from CSV", transactions.len());
    println!();

    // Show first 3 transactions
    println!("Sample Transactions:");
    for (i, tx) in transactions.iter().take(3).enumerate() {
        println!("  {}. {} - {} items", i + 1, tx.id, tx.items.len());
        println!("     Items: {}", tx.items.join(", "));
        println!("     Time: {}", tx.timestamp.format("%Y-%m-%d %H:%M:%S"));
    }
    println!("  ... and {} more", transactions.len() - 3);
    println!();

    Ok(())
}

fn demo_csv_mining() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 2: Mining Rules from CSV Data ---");

    let file_path = "/tmp/sample_transactions.csv";

    // Load transactions
    let transactions = DataLoader::from_csv(file_path)?;

    // Mining configuration
    let config = MiningConfig {
        min_support: 0.2,    // 20% - at least 3 out of 15 transactions
        min_confidence: 0.7, // 70% confidence
        min_lift: 1.2,       // Positive correlation
        ..Default::default()
    };

    println!("Mining Configuration:");
    println!("  Min Support: {:.0}%", config.min_support * 100.0);
    println!("  Min Confidence: {:.0}%", config.min_confidence * 100.0);
    println!("  Min Lift: {:.1}", config.min_lift);
    println!();

    // Mine association rules
    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;

    println!("Mining rules...");
    let rules = miner.mine_association_rules()?;

    println!("✓ Found {} association rules", rules.len());
    println!();

    // Display rules
    if !rules.is_empty() {
        println!("Top {} Rules:", rules.len().min(10));
        for (idx, rule) in rules.iter().take(10).enumerate() {
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

        // Rule quality summary
        let avg_conf = rules.iter().map(|r| r.metrics.confidence).sum::<f64>() / rules.len() as f64;
        let avg_lift = rules.iter().map(|r| r.metrics.lift).sum::<f64>() / rules.len() as f64;

        println!("Rule Quality Summary:");
        println!("  Average Confidence: {:.1}%", avg_conf * 100.0);
        println!("  Average Lift: {:.2}", avg_lift);
        println!();

        // Generate GRL
        use rust_rule_miner::export::GrlExporter;
        let grl = GrlExporter::to_grl(&rules);
        let grl_path = "/tmp/csv_mined_rules.grl";
        fs::write(grl_path, &grl)?;

        println!("✓ Generated GRL file: {}", grl_path);
        println!("  {} lines of code", grl.lines().count());
        println!("  {} bytes", grl.len());
    } else {
        println!("ℹ️  No rules found matching the criteria");
        println!("  Try lowering min_support or min_confidence");
    }

    println!();

    Ok(())
}

fn cleanup_temp_files() {
    println!("--- Cleanup ---");
    fs::remove_file("/tmp/sample_transactions.csv").ok();
    fs::remove_file("/tmp/csv_mined_rules.grl").ok();
    println!("✓ Temporary files removed");
}
