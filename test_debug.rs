use rust_rule_miner::{MiningConfig, RuleMiner, Transaction};
use chrono::Utc;

fn main() {
    let transactions = vec![
        Transaction::new("tx1", vec!["A".to_string(), "B".to_string()], Utc::now()),
        Transaction::new("tx2", vec!["A".to_string(), "B".to_string()], Utc::now()),
        Transaction::new("tx3", vec!["A".to_string(), "B".to_string()], Utc::now()),
    ];

    let config = MiningConfig {
        min_support: 0.5,
        min_confidence: 0.8,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions).unwrap();
    let rules = miner.mine_association_rules().unwrap();
    
    println!("Total rules mined: {}", rules.len());
    for (i, rule) in rules.iter().enumerate() {
        println!("Rule {}: {:?} => {:?} (conf: {:.2}, support: {:.2})", 
            i, rule.antecedent, rule.consequent, rule.metrics.confidence, rule.metrics.support);
    }
}
