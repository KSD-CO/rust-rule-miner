//! PostgreSQL Streaming + Rule Mining Example
//!
//! This example demonstrates:
//! 1. Streaming transaction data from PostgreSQL database
//! 2. Mining association rules from the streamed data
//! 3. Executing rules in real-time using rust-rule-engine
//!
//! # Setup
//!
//! 1. Install PostgreSQL and create a database:
//!    ```bash
//!    createdb rule_mining_demo
//!    ```
//!
//! 2. Run the setup script:
//!    ```bash
//!    psql rule_mining_demo < examples/postgres_setup.sql
//!    ```
//!
//! 3. Set the DATABASE_URL environment variable:
//!    ```bash
//!    export DATABASE_URL="postgresql://username:password@localhost/rule_mining_demo"
//!    ```
//!
//! 4. Run this example:
//!    ```bash
//!    cargo run --example postgres_stream_mining --features "postgres,engine"
//!    ```

use chrono::{DateTime, Utc};
use rust_rule_miner::{
    engine::{facts_from_cart, MiningRuleEngine},
    MiningConfig, RuleMiner, Transaction,
};
use std::env;
use tokio_postgres::{NoTls, Row};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PostgreSQL Streaming + Rule Mining Demo ===\n");

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:postgres@localhost/rule_mining_demo".to_string()
    });

    println!("📊 Connecting to PostgreSQL database...");
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

    // Spawn connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    println!("✓ Connected successfully\n");

    // ========== STEP 1: Stream Transactions from PostgreSQL ==========
    println!("STEP 1: Streaming transactions from database...\n");

    let query = r#"
        SELECT
            t.transaction_id,
            t.customer_id,
            t.transaction_date,
            array_agg(ti.product_name ORDER BY ti.product_name) as items
        FROM transactions t
        JOIN transaction_items ti ON t.transaction_id = ti.transaction_id
        GROUP BY t.transaction_id, t.customer_id, t.transaction_date
        ORDER BY t.transaction_date
    "#;

    let rows = client.query(query, &[]).await?;
    println!("✓ Retrieved {} transactions", rows.len());

    // Convert database rows to Transaction objects
    let mut transactions = Vec::new();
    for row in rows {
        let transaction = row_to_transaction(&row)?;
        println!(
            "  - {} ({} items): {:?}",
            transaction.id,
            transaction.items.len(),
            transaction.items
        );
        transactions.push(transaction);
    }
    println!();

    // ========== STEP 2: Mine Association Rules ==========
    println!("STEP 2: Mining association rules from transactions...\n");

    let config = MiningConfig {
        min_support: 0.3,
        min_confidence: 0.6,
        min_lift: 1.2,
        ..Default::default()
    };

    println!("Configuration:");
    println!("  - Min Support: {}", config.min_support);
    println!("  - Min Confidence: {}", config.min_confidence);
    println!("  - Min Lift: {}", config.min_lift);
    println!();

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;
    let rules = miner.mine_association_rules()?;

    println!("✓ Discovered {} association rules:", rules.len());
    for (i, rule) in rules.iter().enumerate() {
        println!(
            "  {}. {:?} => {:?}",
            i + 1,
            rule.antecedent,
            rule.consequent
        );
        println!(
            "     Support: {:.1}%, Confidence: {:.1}%, Lift: {:.2}",
            rule.metrics.support * 100.0,
            rule.metrics.confidence * 100.0,
            rule.metrics.lift
        );
    }
    println!();

    // ========== STEP 3: Load Rules into Engine ==========
    println!("STEP 3: Loading rules into rust-rule-engine...\n");

    let mut engine = MiningRuleEngine::new("PostgresMiningRules");
    let loaded = engine.load_rules(&rules)?;
    println!("✓ Loaded {} rules into engine\n", loaded);

    // ========== STEP 4: Test Real-time Recommendations ==========
    println!("STEP 4: Testing real-time product recommendations...\n");

    // Test case 1: Customer adds Laptop to cart
    test_recommendation(
        &mut engine,
        "Test 1: Customer adds Laptop",
        vec!["Laptop".to_string()],
    )?;

    // Test case 2: Customer adds Phone to cart
    test_recommendation(
        &mut engine,
        "Test 2: Customer adds Phone",
        vec!["Phone".to_string()],
    )?;

    // Test case 3: Customer adds Monitor to cart
    test_recommendation(
        &mut engine,
        "Test 3: Customer adds Monitor",
        vec!["Monitor".to_string()],
    )?;

    // Test case 4: Customer adds Laptop and Mouse to cart
    test_recommendation(
        &mut engine,
        "Test 4: Customer adds Laptop + Mouse",
        vec!["Laptop".to_string(), "Mouse".to_string()],
    )?;

    // ========== STEP 5: Stream New Transactions (Simulation) ==========
    println!("STEP 5: Simulating real-time transaction stream...\n");

    let new_transactions = vec![
        ("New Customer 1", vec!["Laptop", "Mouse"]),
        ("New Customer 2", vec!["Phone", "Phone Case"]),
        ("New Customer 3", vec!["Monitor", "Webcam"]),
    ];

    for (customer, items) in new_transactions {
        println!("📦 New transaction from {}", customer);
        println!("   Items in cart: {:?}", items);

        let cart_items: Vec<String> = items.iter().map(|s| s.to_string()).collect();
        let facts = facts_from_cart(cart_items.clone());
        let result = engine.execute(&facts)?;

        if result.has_fired() {
            if let Some(recommendations) = result.get("Recommendation.items") {
                println!("   💡 Recommendations generated: {:?}", recommendations);
            }
        } else {
            println!("   No recommendations (no rules matched)");
        }
        println!();
    }

    // ========== Summary ==========
    println!("=== Summary ===");
    println!(
        "✓ Streamed {} transactions from PostgreSQL",
        miner.transaction_count()
    );
    println!("✓ Mined {} association rules", rules.len());
    println!("✓ Loaded rules into rust-rule-engine");
    println!("✓ Generated real-time recommendations");
    println!();
    println!("This demonstrates the complete workflow:");
    println!("  PostgreSQL → Stream Data → Mine Rules → Load Engine → Real-time Execution");

    Ok(())
}

/// Convert a PostgreSQL row to a Transaction
fn row_to_transaction(row: &Row) -> Result<Transaction, Box<dyn std::error::Error>> {
    let transaction_id: String = row.try_get("transaction_id")?;
    let timestamp: DateTime<Utc> = row.try_get::<_, DateTime<Utc>>("transaction_date")?;
    let items: Vec<String> = row.try_get("items")?;

    Ok(Transaction::new(&transaction_id, items, timestamp))
}

/// Test recommendation for a given shopping cart
fn test_recommendation(
    engine: &mut MiningRuleEngine,
    test_name: &str,
    cart_items: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}:", test_name);
    println!("  Cart: {:?}", cart_items);

    let facts = facts_from_cart(cart_items);
    let result = engine.execute(&facts)?;

    println!("  Rules fired: {}", result.rules_fired);

    if let Some(recommendations) = result.get("Recommendation.items") {
        println!("  Recommendations: {:?}", recommendations);
    } else {
        println!("  Recommendations: None");
    }
    println!();

    Ok(())
}
