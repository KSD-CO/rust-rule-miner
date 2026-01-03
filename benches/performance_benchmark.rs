use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rule_miner::{MiningAlgorithm, MiningConfig, RuleMiner, Transaction};

fn generate_transactions(count: usize) -> Vec<Transaction> {
    let products = vec![
        "Laptop",
        "Mouse",
        "Keyboard",
        "Monitor",
        "USB_Hub",
        "Phone",
        "Case",
        "Protector",
        "Charger",
        "Tablet",
        "Stylus",
        "Headphones",
        "Camera",
        "Lens",
        "Tripod",
        "Card",
    ];

    let mut transactions = Vec::new();
    let base_time = Utc::now();

    for i in 0..count {
        let num_items = (i % 3) + 1;
        let start_idx = (i * 7) % products.len();

        let mut items = Vec::new();
        for j in 0..num_items {
            let idx = (start_idx + j) % products.len();
            items.push(products[idx].to_string());
        }

        let timestamp = base_time + chrono::Duration::seconds(i as i64 * 60);
        transactions.push(Transaction::new(format!("tx{}", i), items, timestamp));
    }

    transactions
}

fn benchmark_apriori(c: &mut Criterion) {
    let mut group = c.benchmark_group("apriori");

    for size in [100, 1_000, 10_000].iter() {
        let transactions = generate_transactions(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut miner = RuleMiner::new(MiningConfig {
                    min_support: 0.05,
                    min_confidence: 0.6,
                    min_lift: 1.2,
                    max_time_gap: None,
                    algorithm: MiningAlgorithm::Apriori,
                });
                miner
                    .add_transactions(black_box(transactions.clone()))
                    .unwrap();
                let _rules = miner.mine_association_rules().unwrap();
            });
        });
    }

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10); // Fewer samples for memory tests

    for size in [1_000, 10_000].iter() {
        let transactions = generate_transactions(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut miner = RuleMiner::new(MiningConfig {
                    min_support: 0.05,
                    min_confidence: 0.6,
                    min_lift: 1.2,
                    max_time_gap: None,
                    algorithm: MiningAlgorithm::Apriori,
                });
                miner
                    .add_transactions(black_box(transactions.clone()))
                    .unwrap();
                let rules = miner.mine_association_rules().unwrap();
                black_box(rules);
            });
        });
    }

    group.finish();
}

fn benchmark_rule_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_generation");

    // Pre-generate transactions
    let transactions = generate_transactions(10_000);

    group.bench_function("10k_transactions", |b| {
        b.iter(|| {
            let mut miner = RuleMiner::new(MiningConfig {
                min_support: 0.01, // Lower threshold for more rules
                min_confidence: 0.5,
                min_lift: 1.1,
                max_time_gap: None,
                algorithm: MiningAlgorithm::Apriori,
            });
            miner
                .add_transactions(black_box(transactions.clone()))
                .unwrap();
            let rules = miner.mine_association_rules().unwrap();
            black_box(rules.len());
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_apriori,
    benchmark_memory_usage,
    benchmark_rule_generation
);
criterion_main!(benches);
