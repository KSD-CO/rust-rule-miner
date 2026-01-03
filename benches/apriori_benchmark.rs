use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_rule_miner::{MiningConfig, RuleMiner, Transaction};

fn create_test_transactions(count: usize) -> Vec<Transaction> {
    let items = ["A", "B", "C", "D", "E", "F", "G", "H"];
    let mut transactions = Vec::new();

    for i in 0..count {
        let num_items = (i % 5) + 2; // 2-6 items per transaction
        let tx_items: Vec<String> = items
            .iter()
            .take(num_items)
            .map(|s| s.to_string())
            .collect();

        transactions.push(Transaction::new(format!("tx{}", i), tx_items, Utc::now()));
    }

    transactions
}

fn bench_apriori_1k(c: &mut Criterion) {
    c.bench_function("apriori_1k_transactions", |b| {
        let transactions = create_test_transactions(1000);
        let config = MiningConfig::default();

        b.iter(|| {
            let mut miner = RuleMiner::new(config.clone());
            miner.add_transactions(transactions.clone()).unwrap();
            black_box(miner.mine_association_rules().unwrap())
        });
    });
}

fn bench_apriori_100(c: &mut Criterion) {
    c.bench_function("apriori_100_transactions", |b| {
        let transactions = create_test_transactions(100);
        let config = MiningConfig::default();

        b.iter(|| {
            let mut miner = RuleMiner::new(config.clone());
            miner.add_transactions(transactions.clone()).unwrap();
            black_box(miner.mine_association_rules().unwrap())
        });
    });
}

criterion_group!(benches, bench_apriori_100, bench_apriori_1k);
criterion_main!(benches);
