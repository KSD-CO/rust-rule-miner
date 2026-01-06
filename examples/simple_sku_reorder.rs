//! Example: Simple SKU Reorder - Version đơn giản dễ hiểu
//!
//! 📦 BÀI TOÁN: Dự đoán SKU nào cần đặt hàng dựa trên SKU đang bán
//!
//! VÍ DỤ THỰC TẾ:
//! - Khi bán SKU "Táo" → Thường bán kèm "Cam"
//! - => Nếu "Táo" đang bán chạy → Cần đặt hàng thêm "Cam"
//!
//! Run:
//! ```bash
//! cargo run --example simple_sku_reorder
//! ```

use chrono::Timelike;
use rust_rule_miner::{
    data_loader::{ColumnMapping, DataLoader},
    MiningAlgorithm, MiningConfig, RuleMiner,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║          HỆ THỐNG DỰ ĐOÁN SKU CẦN ĐẶT HÀNG (Đơn giản)           ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    println!("📖 GIẢI THÍCH:");
    println!("   Phân tích lịch sử bán hàng để tìm các SKU thường bán cùng nhau");
    println!("   Khi phát hiện SKU A đang bán → Gợi ý đặt hàng SKU B\n");

    // BƯỚC 1: Load dữ liệu
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("BƯỚC 1: ĐỌC DỮ LIỆU");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let csv_path = "examples/buyer_stock.csv";
    let mapping = ColumnMapping::simple(1, 2, 14); // location_id, SKU, date
    let raw_data = DataLoader::from_csv(csv_path, mapping)?;

    println!("✓ Đọc được: {} dòng dữ liệu SKU", raw_data.len());
    println!("  (Mỗi dòng = 1 SKU được cập nhật tại 1 thời điểm)\n");

    // BƯỚC 2: Nhóm SKU theo khung giờ
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("BƯỚC 2: NHÓM CÁC SKU BÁN CÙNG LÚC");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("💡 Ý tưởng: SKU bán trong cùng khung giờ = bán cùng nhau");
    println!("   Khung giờ: 4 tiếng (00-04h, 04-08h, 08-12h, ...)\n");

    let mut time_windows: HashMap<String, Vec<String>> = HashMap::new();

    for record in raw_data {
        if record.items.is_empty() || record.items[0].trim().is_empty() {
            continue;
        }

        let hour = record.timestamp.hour();
        let block = (hour / 4) * 4;
        let time_key = record
            .timestamp
            .format(&format!("%Y-%m-%d {:02}h", block))
            .to_string();

        for sku in record.items {
            let sku = sku.trim().to_string();
            if !sku.is_empty() {
                time_windows.entry(time_key.clone()).or_default().push(sku);
            }
        }
    }

    println!("✓ Nhóm được: {} khung giờ có SKU bán\n", time_windows.len());

    // Tạo transactions
    use chrono::Utc;
    let transactions: Vec<rust_rule_miner::Transaction> = time_windows
        .into_iter()
        .map(|(time, skus)| {
            let mut unique = skus;
            unique.sort();
            unique.dedup();
            rust_rule_miner::Transaction::new(time, unique, Utc::now())
        })
        .filter(|tx| tx.items.len() >= 2 && tx.items.len() <= 30)
        .collect();

    println!(
        "✓ Tạo được: {} transactions (khung giờ có 2-30 SKUs)\n",
        transactions.len()
    );

    // Show sample
    if let Some(sample) = transactions.first() {
        println!("📋 VÍ DỤ 1 TRANSACTION:");
        println!("   Thời gian: {}", sample.id);
        println!("   Các SKU bán cùng lúc ({} SKUs):", sample.items.len());
        for (i, sku) in sample.items.iter().take(5).enumerate() {
            println!("      {}. {}", i + 1, sku);
        }
        if sample.items.len() > 5 {
            println!("      ... và {} SKUs khác", sample.items.len() - 5);
        }
        println!();
    }

    // BƯỚC 3: Tìm patterns
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("BƯỚC 3: TÌM PATTERNS (SKU NÀO BÁN → CẦN ĐẶT SKU NÀO)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("⚙️  Cấu hình tìm kiếm:");
    println!("   • Support >= 20% (pattern xuất hiện ít nhất 20% khung giờ)");
    println!("   • Confidence >= 70% (độ chắc chắn >= 70%)");
    println!("   • Lift >= 2.0 (tương quan mạnh gấp 2 lần)\n");

    let config = MiningConfig {
        min_support: 0.20,
        min_confidence: 0.70,
        min_lift: 2.0,
        algorithm: MiningAlgorithm::FPGrowth,
        ..Default::default()
    };

    let mut miner = RuleMiner::new(config);
    miner.add_transactions(transactions)?;
    let rules = miner.mine_association_rules()?;

    println!("✓ Tìm được: {} patterns đủ điều kiện\n", rules.len());

    if rules.is_empty() {
        println!("⚠️  Không tìm thấy pattern nào!");
        println!("💡 Thử giảm ngưỡng: support < 20%, confidence < 70%");
        return Ok(());
    }

    // BƯỚC 4: Hiển thị kết quả dễ hiểu
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("BƯỚC 4: TOP {} RULES ĐỀ XUẤT ĐẶT HÀNG", rules.len().min(10));
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    for (i, rule) in rules.iter().take(10).enumerate() {
        println!("═══ RULE #{} ═══", i + 1);
        println!();

        // Phần IF (SKU đang bán)
        println!("📌 TÌNH HUỐNG:");
        if rule.antecedent.len() == 1 {
            println!("   Khi SKU \"{}\" đang BÁN CHẠY", rule.antecedent[0]);
        } else {
            println!("   Khi các SKU sau đang BÁN CHẠY:");
            for sku in &rule.antecedent {
                println!("      • {}", sku);
            }
        }
        println!();

        // Phần THEN (SKU cần đặt hàng)
        println!("💡 GỢI Ý:");
        if rule.consequent.len() == 1 {
            println!("   → ĐẶT HÀNG SKU: \"{}\"", rule.consequent[0]);
        } else {
            println!("   → ĐẶT HÀNG CÁC SKU:");
            for sku in &rule.consequent {
                println!("      • {}", sku);
            }
        }
        println!();

        // Metrics giải thích
        println!("📊 CHỈ SỐ:");
        println!("   ✓ Độ tin cậy: {:.0}%", rule.metrics.confidence * 100.0);
        println!(
            "     (Khi IF xảy ra → THEN xảy ra {:.0}% thời gian)",
            rule.metrics.confidence * 100.0
        );
        println!();
        println!("   ✓ Tần suất: {:.0}%", rule.metrics.support * 100.0);
        println!(
            "     (Pattern này xuất hiện {:.0}% khung giờ)",
            rule.metrics.support * 100.0
        );
        println!();
        println!("   ✓ Độ mạnh: {:.1}x", rule.metrics.lift);
        println!(
            "     (Bán cùng nhau mạnh gấp {:.1} lần ngẫu nhiên)",
            rule.metrics.lift
        );
        println!();

        // Đánh giá
        let score = rule.metrics.confidence * rule.metrics.lift;
        if score > 4.0 {
            println!("⭐ ĐỘ ƯU TIÊN: CAO (Nên đặt hàng ngay!)");
        } else if score > 2.5 {
            println!("⭐ ĐỘ ƯU TIÊN: TRUNG BÌNH");
        } else {
            println!("⭐ ĐỘ ƯU TIÊN: THẤP");
        }

        println!("\n{}\n", "─".repeat(70));
    }

    // Tóm tắt
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("TÓM TẮT");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let high_priority = rules
        .iter()
        .filter(|r| r.metrics.confidence * r.metrics.lift > 4.0)
        .count();
    let avg_conf = rules.iter().map(|r| r.metrics.confidence).sum::<f64>() / rules.len() as f64;

    println!("📊 Tổng số rules: {}", rules.len());
    println!(
        "⭐ Rules ưu tiên cao: {} ({:.0}%)",
        high_priority,
        high_priority as f64 / rules.len() as f64 * 100.0
    );
    println!("📈 Độ tin cậy trung bình: {:.0}%", avg_conf * 100.0);

    // Unique SKUs
    let mut trigger_skus = std::collections::HashSet::new();
    let mut reorder_skus = std::collections::HashSet::new();
    for rule in &rules {
        trigger_skus.extend(rule.antecedent.clone());
        reorder_skus.extend(rule.consequent.clone());
    }

    println!("\n🎯 Insight:");
    println!(
        "   • {} SKU khác nhau làm \"trigger\" (dấu hiệu)",
        trigger_skus.len()
    );
    println!("   • {} SKU khác nhau cần \"đặt hàng\"", reorder_skus.len());

    println!("\n💡 CÁCH SỬ DỤNG:");
    println!("   1. Khi thấy SKU trong phần \"TÌNH HUỐNG\" bán chạy");
    println!("   2. → Kiểm tra kho SKU trong phần \"GỢI Ý\"");
    println!("   3. → Nếu sắp hết → Đặt hàng ngay!");
    println!("   4. → Ưu tiên rules có độ ưu tiên CAO\n");

    println!("✅ Hoàn tất! Dùng các rules trên để quyết định đặt hàng.\n");

    Ok(())
}
