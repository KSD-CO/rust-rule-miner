/// Demonstrates high-performance data loading with excelstream
///
/// excelstream provides automatic streaming with constant memory usage
/// (~3-35 MB) regardless of file size - perfect for large datasets.
use rust_rule_miner::{
    data_loader::DataLoader, export::GrlExporter, MiningAlgorithm, MiningConfig, RuleMiner,
};
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("excelstream Integration Demo");
    println!("{}", "=".repeat(60));
    println!();

    // Create a large temporary CSV file
    let csv_path = "/tmp/large_sales_data.csv";
    create_large_csv(csv_path, 10_000)?;
    println!("Created test dataset with 10,000 transactions");
    println!("File size: ~{} KB", fs::metadata(csv_path)?.len() / 1024);
    println!();

    // Load and mine with excelstream (automatic streaming)
    println!("Mining with excelstream");
    println!("{}", "-".repeat(60));
    let start = std::time::Instant::now();

    let mut miner = RuleMiner::new(MiningConfig {
        min_support: 0.01,
        min_confidence: 0.6,
        min_lift: 1.2,
        max_time_gap: None,
        algorithm: MiningAlgorithm::Apriori,
    });

    // DataLoader::from_csv uses excelstream internally for streaming
    // NOTE: For multi-field data, use DataLoader::from_csv_with_mapping() with ColumnMapping
    // Example: DataLoader::from_csv_with_mapping(path, ColumnMapping::simple(0, 1, 5))
    // See examples/04_load_from_excel_csv.rs for complete column mapping examples
    println!("Loading transactions...");
    let transactions = DataLoader::from_csv(csv_path)?;
    println!("Loaded {} transactions", transactions.len());
    println!("Memory during load: ~3-35 MB (constant!)");
    println!();

    miner.add_transactions(transactions)?;

    println!("Mining association rules...");
    let rules = miner.mine_association_rules()?;
    let elapsed = start.elapsed();
    println!("Found {} rules in {:.2?}", rules.len(), elapsed);
    println!();

    // Export rules to GRL
    println!("Exporting to GRL");
    println!("{}", "-".repeat(60));
    let grl = GrlExporter::to_grl(&rules);
    let grl_path = "/tmp/excelstream_mined_rules.grl";
    fs::write(grl_path, &grl)?;
    println!("Exported {} rules to {}", rules.len(), grl_path);
    println!();

    // Show top 5 rules
    println!("Top 5 Rules (by confidence)");
    println!("{}", "-".repeat(60));
    let mut top_rules = rules.clone();
    top_rules.sort_by(|a, b| {
        b.metrics
            .confidence
            .partial_cmp(&a.metrics.confidence)
            .unwrap()
    });

    for (i, rule) in top_rules.iter().take(5).enumerate() {
        println!("{}. {:?} => {:?}", i + 1, rule.antecedent, rule.consequent);
        println!("   Confidence: {:.1}%", rule.metrics.confidence * 100.0);
        println!("   Support: {:.1}%", rule.metrics.support * 100.0);
        println!("   Lift: {:.2}", rule.metrics.lift);
        println!();
    }

    // Performance Analysis
    println!("Performance Analysis");
    println!("{}", "-".repeat(60));
    println!("Dataset size: 10,000 transactions");
    println!("Memory usage: ~3-35 MB (constant, thanks to excelstream!)");
    println!("Total time: {:.2?}", elapsed);
    println!();

    println!("Key Benefits of excelstream:");
    println!("- Constant memory usage regardless of file size");
    println!("- Handles Excel (.xlsx) and CSV files");
    println!("- Ultra-fast streaming performance");
    println!("- Perfect for large datasets (1M+ transactions)");
    println!();

    // Cleanup
    fs::remove_file(csv_path).ok();
    println!("Cleanup complete");

    Ok(())
}

/// Create a large CSV file for testing
fn create_large_csv(path: &str, num_transactions: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::File::create(path)?;

    // Header
    writeln!(file, "transaction_id,items,timestamp")?;

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
        "TabletCase",
        "Stylus",
        "Headphones",
        "Camera",
        "Lens",
        "Tripod",
        "Card",
    ];

    use chrono::Utc;
    let base_time = Utc::now();

    for i in 0..num_transactions {
        // Generate random product combinations
        let num_items = (i % 3) + 1; // 1-3 items per transaction
        let start_idx = (i * 7) % products.len();

        let mut items = Vec::new();
        for j in 0..num_items {
            let idx = (start_idx + j) % products.len();
            items.push(products[idx]);
        }

        let timestamp = base_time + chrono::Duration::seconds(i as i64 * 60);

        writeln!(
            file,
            "tx{},{},{}",
            i,
            items.join(","),
            timestamp.to_rfc3339()
        )?;
    }

    Ok(())
}
