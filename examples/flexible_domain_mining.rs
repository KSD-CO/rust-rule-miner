//! Flexible Domain Mining Example
//!
//! Demonstrates how to use rust-rule-miner with custom field names
//! for different business domains beyond shopping carts.
//!
//! # Domains covered:
//! - E-commerce (Shopping Cart)
//! - Fraud Detection (Transaction Analysis)
//! - Content Recommendation (User Behavior)
//! - Security (Access Pattern Analysis)
//!
//! # Run:
//! ```bash
//! cargo run --example flexible_domain_mining --features "engine"
//! ```

use chrono::Utc;
use rust_rule_engine::Value;
use rust_rule_miner::{
    engine::{facts_from_items, facts_from_items_with_metadata, MiningRuleEngine},
    export::GrlConfig,
    MiningConfig, RuleMiner, Transaction,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Flexible Domain Mining Examples ===\n");

    // ========== DOMAIN 1: E-COMMERCE ==========
    println!("🛒 DOMAIN 1: E-COMMERCE (Product Recommendations)\n");
    ecommerce_example()?;

    // ========== DOMAIN 2: FRAUD DETECTION ==========
    println!("\n🔒 DOMAIN 2: FRAUD DETECTION (Suspicious Patterns)\n");
    fraud_detection_example()?;

    // ========== DOMAIN 3: CONTENT RECOMMENDATION ==========
    println!("\n📺 DOMAIN 3: CONTENT RECOMMENDATION (Video/Article Suggestions)\n");
    content_recommendation_example()?;

    // ========== DOMAIN 4: SECURITY ANALYSIS ==========
    println!("\n🛡️  DOMAIN 4: SECURITY (Access Pattern Analysis)\n");
    security_analysis_example()?;

    Ok(())
}

/// E-commerce: Product recommendation system
fn ecommerce_example() -> Result<(), Box<dyn std::error::Error>> {
    // Historical purchase data
    let transactions = vec![
        Transaction::new(
            "order1",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "order2",
            vec![
                "Laptop".to_string(),
                "Mouse".to_string(),
                "Keyboard".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "order3",
            vec!["Laptop".to_string(), "USB Hub".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "order4",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
    ];

    // Mine rules
    let config = MiningConfig {
        min_support: 0.5,
        min_confidence: 0.75,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;
    let rules = miner.mine_association_rules()?;

    println!("  Mined {} product association rules", rules.len());

    // Configure for shopping cart
    let grl_config = GrlConfig::custom("Cart.products", "Recommendations.items");

    let mut engine = MiningRuleEngine::with_config("EcommerceRules", grl_config.clone());
    engine.load_rules(&rules)?;

    // Test recommendation
    let customer_cart = vec!["Laptop".to_string()];
    println!("  Customer adds: {:?}", customer_cart);

    let facts = facts_from_items(customer_cart, &grl_config);
    let result = engine.execute(&facts)?;

    if let Some(recommendations) = result.get("Recommendations.items") {
        println!("  ✓ Recommendations: {:?}", recommendations);
    }

    Ok(())
}

/// Fraud detection: Identify suspicious transaction patterns
fn fraud_detection_example() -> Result<(), Box<dyn std::error::Error>> {
    // Historical fraud patterns
    let fraud_patterns = vec![
        Transaction::new(
            "fraud1",
            vec!["Multiple_Countries".to_string(), "Large_Amount".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "fraud2",
            vec!["Multiple_Countries".to_string(), "New_Device".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "fraud3",
            vec![
                "Multiple_Countries".to_string(),
                "Unusual_Time".to_string(),
                "Large_Amount".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "fraud4",
            vec!["Multiple_Countries".to_string(), "Large_Amount".to_string()],
            Utc::now(),
        ),
    ];

    let config = MiningConfig {
        min_support: 0.5,
        min_confidence: 0.7,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(fraud_patterns)?;
    let rules = miner.mine_association_rules()?;

    println!("  Mined {} fraud pattern rules", rules.len());

    // Configure for fraud detection
    let grl_config = GrlConfig::custom("Transaction.indicators", "FraudAlert.flags");

    let mut engine = MiningRuleEngine::with_config("FraudDetection", grl_config.clone());
    engine.load_rules(&rules)?;

    // Analyze a transaction
    let transaction_indicators = vec!["Multiple_Countries".to_string(), "Large_Amount".to_string()];
    println!("  Transaction indicators: {:?}", transaction_indicators);

    let metadata = vec![
        (
            "Transaction.amount".to_string(),
            Value::String("10000.0".to_string()),
        ),
        (
            "Transaction.risk_score".to_string(),
            Value::String("high".to_string()),
        ),
    ];

    let facts = facts_from_items_with_metadata(transaction_indicators, &grl_config, Some(metadata));

    let result = engine.execute(&facts)?;

    if result.has_fired() {
        if let Some(alerts) = result.get("FraudAlert.flags") {
            println!("  ⚠️  Fraud alerts triggered: {:?}", alerts);
            println!("  ⚠️  Manual review recommended!");
        }
    } else {
        println!("  ✓ Transaction appears normal");
    }

    Ok(())
}

/// Content recommendation: Video/Article suggestions
fn content_recommendation_example() -> Result<(), Box<dyn std::error::Error>> {
    // User viewing history patterns
    let viewing_history = vec![
        Transaction::new(
            "user1",
            vec!["Tech".to_string(), "AI".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "user2",
            vec!["Tech".to_string(), "AI".to_string(), "ML".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "user3",
            vec!["Tech".to_string(), "Blockchain".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "user4",
            vec!["Tech".to_string(), "AI".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "user5",
            vec!["Tech".to_string(), "AI".to_string(), "Python".to_string()],
            Utc::now(),
        ),
    ];

    let config = MiningConfig {
        min_support: 0.4,
        min_confidence: 0.6,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(viewing_history)?;
    let rules = miner.mine_association_rules()?;

    println!("  Mined {} content association rules", rules.len());

    // Configure for content recommendation
    let grl_config = GrlConfig::custom("UserHistory.topics", "ContentSuggestions.topics");

    let mut engine = MiningRuleEngine::with_config("ContentRecommendation", grl_config.clone());
    engine.load_rules(&rules)?;

    // User's current interests
    let user_interests = vec!["Tech".to_string()];
    println!("  User interested in: {:?}", user_interests);

    let facts = facts_from_items(user_interests, &grl_config);
    let result = engine.execute(&facts)?;

    if let Some(suggestions) = result.get("ContentSuggestions.topics") {
        println!("  ✓ Suggested topics: {:?}", suggestions);
    }

    Ok(())
}

/// Security: Access pattern analysis
fn security_analysis_example() -> Result<(), Box<dyn std::error::Error>> {
    // Historical attack patterns
    let attack_patterns = vec![
        Transaction::new(
            "attack1",
            vec!["Port_Scan".to_string(), "SQL_Injection".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "attack2",
            vec!["Port_Scan".to_string(), "Brute_Force".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "attack3",
            vec![
                "Port_Scan".to_string(),
                "SQL_Injection".to_string(),
                "Data_Exfil".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "attack4",
            vec!["Port_Scan".to_string(), "SQL_Injection".to_string()],
            Utc::now(),
        ),
    ];

    let config = MiningConfig {
        min_support: 0.5,
        min_confidence: 0.75,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(attack_patterns)?;
    let rules = miner.mine_association_rules()?;

    println!("  Mined {} attack pattern rules", rules.len());

    // Configure for security analysis
    let grl_config = GrlConfig::custom("NetworkActivity.events", "SecurityAlert.threats");

    let mut engine = MiningRuleEngine::with_config("SecurityAnalysis", grl_config.clone());
    engine.load_rules(&rules)?;

    // Analyze network activity
    let detected_events = vec!["Port_Scan".to_string()];
    println!("  Detected network events: {:?}", detected_events);

    let metadata = vec![
        (
            "NetworkActivity.source_ip".to_string(),
            Value::String("192.168.1.100".to_string()),
        ),
        (
            "NetworkActivity.severity".to_string(),
            Value::String("Medium".to_string()),
        ),
    ];

    let facts = facts_from_items_with_metadata(detected_events, &grl_config, Some(metadata));

    let result = engine.execute(&facts)?;

    if result.has_fired() {
        if let Some(threats) = result.get("SecurityAlert.threats") {
            println!("  🚨 Security threats predicted: {:?}", threats);
            println!("  🚨 Increase monitoring level!");
        }
    } else {
        println!("  ✓ No immediate threats detected");
    }

    Ok(())
}
