use chrono::Utc;
use rust_rule_engine::rete::{
    facts::{FactValue, TypedFacts},
    grl_loader::GrlReteLoader,
    propagation::IncrementalEngine,
};
use rust_rule_miner::{export::GrlExporter, MiningConfig, RuleMiner, Transaction};
use std::fs;

// NOTE: This example shows RETE engine integration (high performance).
// RETE is recommended for production with many rules (>100).
//
// For custom field names (not just ShoppingCart.items), use GrlConfig:
//   use rust_rule_miner::export::GrlConfig;
//   let config = GrlConfig::custom("Order.items", "Recommendations.products");
//   let grl = GrlExporter::to_grl_with_config(&rules, &config);
//
// See also: examples/flexible_domain_mining.rs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Integration with RETE Engine: Mine Rules → Execute ===\n");
    println!("NOTE: RETE engine is optimized for high performance with many rules.\n");

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

    // Save to file for RETE loader
    let grl_file = "/tmp/mined_rules.grl";
    fs::write(grl_file, &grl_code)?;
    println!("✓ GRL code saved to {}\n", grl_file);
    println!("{}\n", grl_code);

    // ========== STEP 3: Load Rules into RETE Engine ==========
    println!("STEP 3: Loading mined rules into RETE engine...\n");

    let mut engine = IncrementalEngine::new();
    let rule_count = GrlReteLoader::load_from_file(grl_file, &mut engine)?;

    println!("✓ {} rules loaded successfully\n", rule_count);

    // ========== STEP 4: Test Recommendations ==========
    println!("STEP 4: Testing recommendations with new customer data...\n");

    // Test Case 1: Customer adds Laptop to cart
    println!("Test Case 1: Customer adds Laptop to shopping cart");

    let mut shopping_cart = TypedFacts::new();
    shopping_cart.set(
        "items",
        FactValue::Array(vec![FactValue::String("Laptop".to_string())]),
    );

    let mut recommendations = TypedFacts::new();
    recommendations.set("items", FactValue::Array(vec![]));

    println!("  Before execution:");
    println!("    ShoppingCart.items: [\"Laptop\"]");
    println!("    Recommendation.items: []");

    engine.insert("ShoppingCart".to_string(), shopping_cart);
    engine.insert("Recommendation".to_string(), recommendations.clone());

    let fired = engine.fire_all();
    println!("  After execution:");
    println!("    Rules fired: {}", fired.len());

    // Get updated recommendations from working memory
    let wm = engine.working_memory();
    let rec_facts = wm.get_by_type("Recommendation");
    if !rec_facts.is_empty() {
        if let Some(FactValue::Array(items)) = rec_facts[0].data.get("items") {
            print!("    Recommendation.items: [");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                if let FactValue::String(s) = item {
                    print!("\"{}\"", s);
                }
            }
            println!("]");
        }
    } else {
        println!("    Recommendation.items: (not found)");
    }
    println!();

    // Test Case 2: Customer adds Phone to cart
    println!("Test Case 2: Customer adds Phone to shopping cart");

    // Create new engine for clean test
    let mut engine2 = IncrementalEngine::new();
    GrlReteLoader::load_from_file(grl_file, &mut engine2)?;

    let mut shopping_cart2 = TypedFacts::new();
    shopping_cart2.set(
        "items",
        FactValue::Array(vec![FactValue::String("Phone".to_string())]),
    );

    let mut recommendations2 = TypedFacts::new();
    recommendations2.set("items", FactValue::Array(vec![]));

    println!("  Before execution:");
    println!("    ShoppingCart.items: [\"Phone\"]");
    println!("    Recommendation.items: []");

    engine2.insert("ShoppingCart".to_string(), shopping_cart2);
    engine2.insert("Recommendation".to_string(), recommendations2);

    let fired2 = engine2.fire_all();
    println!("  After execution:");
    println!("    Rules fired: {}", fired2.len());

    let wm2 = engine2.working_memory();
    let rec_facts2 = wm2.get_by_type("Recommendation");
    if !rec_facts2.is_empty() {
        if let Some(FactValue::Array(items)) = rec_facts2[0].data.get("items") {
            print!("    Recommendation.items: [");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                if let FactValue::String(s) = item {
                    print!("\"{}\"", s);
                }
            }
            println!("]");
        }
    } else {
        println!("    Recommendation.items: (not found)");
    }
    println!();

    // ========== Summary ==========
    println!("=== Summary ===");
    println!(
        "✓ Mined {} association rules from historical data",
        rules.len()
    );
    println!("✓ Generated GRL code compatible with RETE engine");
    println!("✓ Loaded and executed rules successfully");
    println!("✓ Recommendations working with RETE incremental engine!");
    println!();
    println!("This demonstrates the complete workflow:");
    println!("  Historical Data → Rule Mining → GRL Generation → RETE Execution → Recommendations");

    Ok(())
}
