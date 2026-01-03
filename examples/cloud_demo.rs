/// Cloud Storage Integration Demo
///
/// Demonstrates loading transactions from:
/// - AWS S3 buckets
/// - HTTP endpoints
///
/// To run this example:
/// ```bash
/// cargo run --example cloud_demo --features cloud
/// ```
///
/// Prerequisites:
/// - AWS credentials configured (for S3)
/// - Network access (for HTTP)

#[cfg(feature = "cloud")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Cloud Storage Integration Demo");
    println!("{}", "=".repeat(60));
    println!();

    // Example 1: Load from AWS S3
    println!("Example 1: AWS S3");
    println!("{}", "-".repeat(60));
    println!("Loading from S3 would look like:");
    println!();
    println!("let transactions = DataLoader::from_s3(");
    println!("    \"my-data-bucket\",");
    println!("    \"sales/2024/transactions.xlsx\",");
    println!("    \"us-east-1\",");
    println!("    0  // sheet index");
    println!(").await?;");
    println!();
    println!("Features:");
    println!("- Streams directly from S3");
    println!("- Constant memory usage (~3-35 MB)");
    println!("- No local disk space required");
    println!("- Works with Excel and CSV files");
    println!();

    // Example 2: Load from HTTP endpoint
    println!("Example 2: HTTP Endpoint");
    println!("{}", "-".repeat(60));
    println!("Loading from HTTP would look like:");
    println!();
    println!("let transactions = DataLoader::from_http(");
    println!("    \"https://example.com/data/transactions.csv\"");
    println!(").await?;");
    println!();
    println!("Features:");
    println!("- Fetches CSV from any HTTP/HTTPS URL");
    println!("- Memory efficient parsing");
    println!("- Perfect for API endpoints");
    println!();

    // Example 3: Complete workflow
    println!("Example 3: Complete Cloud Workflow");
    println!("{}", "-".repeat(60));
    println!(
        "
// 1. Load from cloud storage
let transactions = DataLoader::from_s3(
    \"analytics-bucket\",
    \"daily-sales.xlsx\",
    \"us-west-2\",
    0
).await?;

// 2. Mine patterns
let mut miner = RuleMiner::new(MiningConfig {{
    min_support: 0.05,
    min_confidence: 0.7,
    ..Default::default()
}});
miner.add_transactions(transactions)?;
let rules = miner.mine_association_rules()?;

// 3. Export to GRL
let grl = GrlExporter::to_grl(&rules);
std::fs::write(\"mined_rules.grl\", grl)?;

// 4. Upload results back to S3 (optional)
// S3ExcelWriter::upload(\"results-bucket\", \"rules.grl\", &grl).await?;
"
    );

    println!();
    println!("Use Cases:");
    println!("{}", "-".repeat(60));
    println!("1. Serverless Data Mining");
    println!("   - Lambda functions processing S3 data");
    println!("   - No local disk required");
    println!();
    println!("2. Distributed Analytics");
    println!("   - Multiple workers processing different S3 objects");
    println!("   - Scale horizontally");
    println!();
    println!("3. API-based Pipelines");
    println!("   - Fetch data from REST APIs");
    println!("   - Process and return results");
    println!();
    println!("4. Multi-Cloud Deployments");
    println!("   - Read from AWS S3");
    println!("   - Write to Google Cloud Storage");
    println!("   - Process in Azure Functions");
    println!();

    println!("Configuration:");
    println!("{}", "-".repeat(60));
    println!("AWS S3 Authentication:");
    println!("- Environment variables: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY");
    println!("- IAM role (for EC2/Lambda)");
    println!("- AWS credentials file (~/.aws/credentials)");
    println!();
    println!("HTTP Authentication:");
    println!("- Add headers via reqwest (future enhancement)");
    println!("- API keys in URL parameters");
    println!();

    Ok(())
}

#[cfg(not(feature = "cloud"))]
fn main() {
    println!("This example requires the 'cloud' feature.");
    println!();
    println!("Run with:");
    println!("  cargo run --example cloud_demo --features cloud");
    println!();
    println!("To add cloud support to your project:");
    println!("  rust-rule-miner = {{ version = \"0.1\", features = [\"cloud\"] }}");
}
