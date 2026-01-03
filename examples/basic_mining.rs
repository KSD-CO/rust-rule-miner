use chrono::Utc;
use rust_rule_miner::{export::GrlExporter, MiningConfig, RuleMiner, Transaction};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Rule Mining Example ===\n");

    // 1. Create sample transactions (e-commerce purchases)
    let transactions = vec![
        Transaction::new(
            "tx1",
            vec![
                "Laptop".to_string(),
                "Mouse".to_string(),
                "Keyboard".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "tx2",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx3",
            vec![
                "Laptop".to_string(),
                "Mouse".to_string(),
                "USB-C Hub".to_string(),
            ],
            Utc::now(),
        ),
        Transaction::new(
            "tx4",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx5",
            vec!["Phone".to_string(), "Phone Case".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx6",
            vec!["Phone".to_string(), "Phone Case".to_string()],
            Utc::now(),
        ),
        Transaction::new(
            "tx7",
            vec![
                "Laptop".to_string(),
                "Mouse".to_string(),
                "Laptop Bag".to_string(),
            ],
            Utc::now(),
        ),
    ];

    println!("Loaded {} transactions\n", transactions.len());

    // 2. Configure mining parameters
    let config = MiningConfig {
        min_support: 0.3,    // 30% - pattern must appear in at least 30% of transactions
        min_confidence: 0.7, // 70% - rule must be correct at least 70% of the time
        min_lift: 1.2,       // 20% above random chance
        max_time_gap: None,
        algorithm: rust_rule_miner::MiningAlgorithm::Apriori,
    };

    println!("Mining Configuration:");
    println!("  Min Support: {:.1}%", config.min_support * 100.0);
    println!("  Min Confidence: {:.1}%", config.min_confidence * 100.0);
    println!("  Min Lift: {:.2}", config.min_lift);
    println!("  Algorithm: Apriori\n");

    // 3. Mine association rules
    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;

    println!("Mining association rules...\n");
    let rules = miner.mine_association_rules()?;

    // 4. Display discovered rules
    println!("=== Discovered Rules ({}) ===\n", rules.len());

    for (idx, rule) in rules.iter().enumerate() {
        println!("Rule #{}:", idx + 1);
        println!("  Pattern: {:?} => {:?}", rule.antecedent, rule.consequent);
        println!("  Confidence: {:.1}%", rule.metrics.confidence * 100.0);
        println!("  Support: {:.1}%", rule.metrics.support * 100.0);
        println!("  Lift: {:.2}", rule.metrics.lift);
        println!("  Conviction: {:.2}", rule.metrics.conviction);
        println!("  Quality Score: {:.3}", rule.quality_score());
        println!(
            "  Interpretation: When {} is purchased, {} appears {:.1}% of the time",
            rule.antecedent.join(", "),
            rule.consequent.join(", "),
            rule.metrics.confidence * 100.0
        );
        println!();
    }

    // 5. Generate GRL code
    println!("=== Generated GRL Code ===\n");
    let grl_code = GrlExporter::to_grl(&rules);
    println!("{}", grl_code);

    // 6. Save to file
    std::fs::write("mined_rules.grl", &grl_code)?;
    println!("✓ Rules saved to mined_rules.grl");

    // 7. Show statistics
    println!("\n=== Mining Statistics ===");
    let stats = miner.stats();
    println!("  Frequent Itemsets: {}", stats.frequent_itemsets_count);
    println!("  Rules Generated: {}", stats.rules_generated);

    Ok(())
}
