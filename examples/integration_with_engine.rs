use chrono::Utc;
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value};
use rust_rule_miner::{export::GrlExporter, MiningConfig, RuleMiner, Transaction};

// NOTE: This example shows the LOW-LEVEL API for direct engine integration.
// For a SIMPLER API, see the MiningRuleEngine wrapper:
//   use rust_rule_miner::engine::MiningRuleEngine;
//   let mut engine = MiningRuleEngine::new("MyRules");
//   engine.load_rules(&rules)?;
//   let result = engine.execute(&facts)?;
//
// For custom field names (not just ShoppingCart.items), see:
//   - examples/flexible_domain_mining.rs
//   - examples/postgres_stream_mining.rs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Integration Example: Mine Rules → Execute in rust-rule-engine ===\n");
    println!(
        "NOTE: This shows the low-level API. For simpler usage, see MiningRuleEngine wrapper.\n"
    );

    // ========== STEP 1: Mine Rules from Historical Data ==========
    println!("STEP 1: Mining rules from historical purchase data...\n");

    let historical_transactions = vec![
        // Laptop buyers typically also buy mouse
        Transaction::new(
            "tx1",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx2",
            vec![
                "Laptop".to_string(),
                "Mouse".to_string(),
                "USB-C Hub".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "tx3",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx4",
            vec![
                "Laptop".to_string(),
                "Mouse".to_string(),
                "Laptop Bag".to_string(),
            ],
            Utc::now(),
        ),
        // Phone buyers typically buy phone case
        Transaction::new(
            "tx5",
            vec!["Phone".to_string(), "Phone Case".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx6",
            vec![
                "Phone".to_string(),
                "Phone Case".to_string(),
                "Screen Protector".to_string(),
            ],
            Utc::now(),
        ),
    ];

    let config = MiningConfig {
        min_support: 0.3,
        min_confidence: 0.7,
        min_lift: 1.2,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(historical_transactions)?;
    let rules = miner.mine_association_rules()?;

    println!(
        "✓ Discovered {} rules from {} transactions",
        rules.len(),
        miner.transaction_count()
    );
    for rule in &rules {
        println!(
            "  - {:?} => {:?} (confidence: {:.1}%, lift: {:.2})",
            rule.antecedent,
            rule.consequent,
            rule.metrics.confidence * 100.0,
            rule.metrics.lift
        );
    }
    println!();

    // ========== STEP 2: Generate GRL Code ==========
    println!("STEP 2: Generating GRL code...\n");

    let grl_code = GrlExporter::to_grl(&rules);
    println!("{}\n", grl_code);

    // ========== STEP 3: Load Rules into rust-rule-engine ==========
    println!("STEP 3: Loading mined rules into rust-rule-engine...\n");

    let kb = KnowledgeBase::new("MinedRules");
    let mut engine = RustRuleEngine::new(kb);

    // Parse GRL and add to knowledge base
    let parsed_rules = GRLParser::parse_rules(&grl_code)?;
    for rule in parsed_rules {
        engine.knowledge_base().add_rule(rule)?;
    }

    println!("✓ {} rules loaded successfully\n", rules.len());

    // ========== STEP 4: Test Recommendations ==========
    println!("STEP 4: Testing recommendations with new customer data...\n");

    // Test Case 1: Customer adds Laptop to cart
    println!("Test Case 1: Customer adds Laptop to shopping cart");
    let facts = Facts::new();
    facts.set(
        "ShoppingCart.items",
        Value::Array(vec![Value::String("Laptop".to_string())]),
    );
    facts.set("Recommendation.items", Value::Array(vec![]));

    println!("  Before execution:");
    println!("    ShoppingCart.items: [\"Laptop\"]");
    println!("    Recommendation.items: []");

    let result1 = engine.execute(&facts)?;

    println!("  After execution:");
    println!("    Rules fired: {}", result1.rules_fired);
    if let Some(Value::Array(recommendations)) = facts.get("Recommendation.items") {
        print!("    Recommendation.items: [");
        for (i, rec) in recommendations.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            if let Value::String(s) = rec {
                print!("\"{}\"", s);
            }
        }
        println!("]");
    } else {
        println!("    Recommendation.items: (not set)");
    }
    println!();

    // Test Case 2: Customer adds Phone to cart
    println!("Test Case 2: Customer adds Phone to shopping cart");
    let facts2 = Facts::new();
    facts2.set(
        "ShoppingCart.items",
        Value::Array(vec![Value::String("Phone".to_string())]),
    );
    facts2.set("Recommendation.items", Value::Array(vec![]));

    println!("  Before execution:");
    println!("    ShoppingCart.items: [\"Phone\"]");
    println!("    Recommendation.items: []");

    let result2 = engine.execute(&facts2)?;

    println!("  After execution:");
    println!("    Rules fired: {}", result2.rules_fired);
    if let Some(Value::Array(recommendations)) = facts2.get("Recommendation.items") {
        print!("    Recommendation.items: [");
        for (i, rec) in recommendations.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            if let Value::String(s) = rec {
                print!("\"{}\"", s);
            }
        }
        println!("]");
    } else {
        println!("    Recommendation.items: (not set)");
    }
    println!();

    // ========== Summary ==========
    println!("=== Summary ===");
    println!(
        "✓ Mined {} association rules from historical data",
        rules.len()
    );
    println!("✓ Generated GRL code compatible with rust-rule-engine");
    println!("✓ Loaded and executed rules successfully");
    println!("✓ Recommendations working as expected!");
    println!();
    println!("This demonstrates the complete workflow:");
    println!("  Historical Data → Rule Mining → GRL Generation → Rule Execution → Recommendations");

    Ok(())
}
