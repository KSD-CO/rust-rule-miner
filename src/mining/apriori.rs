use crate::errors::Result;
use crate::transaction::Transaction;
use crate::types::{FrequentItemset, ItemSet};
use std::collections::{HashMap, HashSet};

/// Find all frequent itemsets using Apriori algorithm
pub fn find_frequent_itemsets(
    transactions: &[Transaction],
    min_support: f64,
) -> Result<Vec<FrequentItemset>> {
    let total_transactions = transactions.len() as f64;
    let min_support_count = (min_support * total_transactions).ceil() as usize;

    let mut frequent_itemsets = Vec::new();

    // Level 1: Individual items
    let mut current_level = generate_1_itemsets(transactions);

    while !current_level.is_empty() {
        // Count support for each candidate
        let counts = count_support(transactions, &current_level);

        // Filter by minimum support
        let frequent_k: Vec<_> = counts
            .into_iter()
            .filter(|(_, count)| *count >= min_support_count)
            .collect();

        if frequent_k.is_empty() {
            break;
        }

        // Add to results with support as fraction
        for (itemset, count) in &frequent_k {
            frequent_itemsets.push(FrequentItemset {
                items: itemset.clone(),
                support: *count as f64 / total_transactions,
            });
        }

        // Generate next level candidates (k+1 itemsets from k itemsets)
        current_level = generate_candidates(&frequent_k);
    }

    Ok(frequent_itemsets)
}

/// Generate 1-itemsets (individual items)
fn generate_1_itemsets(transactions: &[Transaction]) -> Vec<ItemSet> {
    let mut items = HashSet::new();

    for tx in transactions {
        for item in &tx.items {
            items.insert(item.clone());
        }
    }

    items.into_iter().map(|item| vec![item]).collect()
}

/// Count support for itemsets
fn count_support(transactions: &[Transaction], itemsets: &[ItemSet]) -> HashMap<ItemSet, usize> {
    let mut counts = HashMap::new();

    for itemset in itemsets {
        let count = transactions
            .iter()
            .filter(|tx| tx.contains_all(itemset))
            .count();
        counts.insert(itemset.clone(), count);
    }

    counts
}

/// Generate (k+1)-itemsets from k-itemsets
fn generate_candidates(frequent_k: &[(ItemSet, usize)]) -> Vec<ItemSet> {
    let mut candidates = Vec::new();

    for i in 0..frequent_k.len() {
        for j in (i + 1)..frequent_k.len() {
            let (set1, _) = &frequent_k[i];
            let (set2, _) = &frequent_k[j];

            // Join if they share k-1 items
            if can_join(set1, set2) {
                let mut new_set = set1.clone();
                // Add the last item from set2 that's not in set1
                if let Some(last_item) = set2.last() {
                    if !new_set.contains(last_item) {
                        new_set.push(last_item.clone());
                        new_set.sort();
                        candidates.push(new_set);
                    }
                }
            }
        }
    }

    // Remove duplicates
    candidates.sort();
    candidates.dedup();

    candidates
}

/// Check if two itemsets can be joined
fn can_join(set1: &[String], set2: &[String]) -> bool {
    if set1.len() != set2.len() {
        return false;
    }

    if set1.is_empty() {
        return false;
    }

    // Check if first k-1 items are the same
    for i in 0..set1.len() - 1 {
        if set1[i] != set2[i] {
            return false;
        }
    }

    // Last items should be different
    set1[set1.len() - 1] != set2[set2.len() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_transactions() -> Vec<Transaction> {
        vec![
            Transaction::new(
                "tx1",
                vec!["A".to_string(), "B".to_string(), "C".to_string()],
                Utc::now(),
            ),
            Transaction::new("tx2", vec!["A".to_string(), "B".to_string()], Utc::now()),
            Transaction::new("tx3", vec!["A".to_string(), "C".to_string()], Utc::now()),
            Transaction::new("tx4", vec!["B".to_string(), "C".to_string()], Utc::now()),
        ]
    }

    #[test]
    fn test_generate_1_itemsets() {
        let transactions = create_test_transactions();
        let itemsets = generate_1_itemsets(&transactions);

        assert_eq!(itemsets.len(), 3); // A, B, C
        assert!(itemsets.contains(&vec!["A".to_string()]));
        assert!(itemsets.contains(&vec!["B".to_string()]));
        assert!(itemsets.contains(&vec!["C".to_string()]));
    }

    #[test]
    fn test_count_support() {
        let transactions = create_test_transactions();
        let itemsets = vec![
            vec!["A".to_string()],
            vec!["B".to_string()],
            vec!["A".to_string(), "B".to_string()],
        ];

        let counts = count_support(&transactions, &itemsets);

        assert_eq!(counts.get(&vec!["A".to_string()]), Some(&3));
        assert_eq!(counts.get(&vec!["B".to_string()]), Some(&3));
        assert_eq!(
            counts.get(&vec!["A".to_string(), "B".to_string()]),
            Some(&2)
        );
    }

    #[test]
    fn test_can_join() {
        let set1 = vec!["A".to_string(), "B".to_string()];
        let set2 = vec!["A".to_string(), "C".to_string()];
        assert!(can_join(&set1, &set2));

        let set3 = vec!["A".to_string(), "B".to_string()];
        let set4 = vec!["C".to_string(), "D".to_string()];
        assert!(!can_join(&set3, &set4));
    }

    #[test]
    fn test_apriori() {
        let transactions = create_test_transactions();
        let frequent = find_frequent_itemsets(&transactions, 0.5).unwrap();

        // Should find: A, B, C (individual items with >= 50% support)
        assert!(frequent.iter().any(|f| f.items == vec!["A".to_string()]));
        assert!(frequent.iter().any(|f| f.items == vec!["B".to_string()]));
        assert!(frequent.iter().any(|f| f.items == vec!["C".to_string()]));

        // Should also find {A,B}, {A,C}, {B,C} (2-itemsets)
        assert!(frequent
            .iter()
            .any(|f| f.items == vec!["A".to_string(), "B".to_string()]));
    }

    #[test]
    fn test_apriori_high_support() {
        let transactions = create_test_transactions();
        let frequent = find_frequent_itemsets(&transactions, 0.75).unwrap();

        // Only A, B, C have >= 75% support (3 out of 4)
        assert!(frequent.iter().any(|f| f.items == vec!["A".to_string()]));
        assert!(frequent.iter().any(|f| f.items == vec!["B".to_string()]));
        assert!(frequent.iter().any(|f| f.items == vec!["C".to_string()]));

        // No 2-itemsets should have >= 75% support
        assert!(frequent.iter().all(|f| f.items.len() == 1));
    }
}
