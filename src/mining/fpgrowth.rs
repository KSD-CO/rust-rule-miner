use crate::errors::Result;
use crate::transaction::Transaction;
use crate::types::{FrequentItemset, ItemSet};
use std::collections::HashMap;

/// Find all frequent itemsets using FP-Growth algorithm
///
/// FP-Growth is more efficient than Apriori for large datasets:
/// - Builds a compact FP-Tree structure
/// - Mines patterns without candidate generation
/// - Better performance for dense datasets
pub fn find_frequent_itemsets(
    transactions: &[Transaction],
    min_support: f64,
) -> Result<Vec<FrequentItemset>> {
    let total_transactions = transactions.len() as f64;
    let min_support_count = (min_support * total_transactions).ceil() as usize;

    // Step 1: Count item frequencies
    let mut item_counts: HashMap<String, usize> = HashMap::new();
    for tx in transactions {
        for item in &tx.items {
            *item_counts.entry(item.clone()).or_insert(0) += 1;
        }
    }

    // Step 2: Filter frequent items (1-itemsets)
    let mut frequent_items: Vec<(String, usize)> = item_counts
        .into_iter()
        .filter(|(_, count)| *count >= min_support_count)
        .collect();

    // Sort by frequency (descending) for FP-Tree efficiency
    frequent_items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    // Create frequency order map for sorting
    let freq_order: HashMap<String, usize> = frequent_items
        .iter()
        .enumerate()
        .map(|(idx, (item, _))| (item.clone(), idx))
        .collect();

    // Step 3: Build FP-Tree
    let mut fp_tree = FPTree::new();
    for tx in transactions {
        // Filter and sort items by frequency order
        let mut ordered_items: Vec<String> = tx
            .items
            .iter()
            .filter(|item| freq_order.contains_key(*item))
            .cloned()
            .collect();

        ordered_items.sort_by_key(|item| freq_order.get(item).unwrap());

        if !ordered_items.is_empty() {
            fp_tree.insert_transaction(&ordered_items);
        }
    }

    // Step 4: Mine patterns from FP-Tree
    let mut frequent_itemsets = Vec::new();

    // Add 1-itemsets
    for (item, count) in &frequent_items {
        frequent_itemsets.push(FrequentItemset {
            items: vec![item.clone()],
            support: *count as f64 / total_transactions,
        });
    }

    // Mine larger itemsets using FP-Growth
    for (item, _) in frequent_items.iter().rev() {
        // Build conditional pattern base
        let conditional_patterns = fp_tree.get_conditional_pattern_base(item);

        if !conditional_patterns.is_empty() {
            // Build conditional FP-Tree
            let mut cond_tree = FPTree::new();
            for (pattern, count) in &conditional_patterns {
                for _ in 0..*count {
                    cond_tree.insert_transaction(pattern);
                }
            }

            // Mine conditional tree
            let cond_patterns =
                mine_conditional_tree(&cond_tree, vec![item.clone()], min_support_count);

            for (itemset, count) in cond_patterns {
                frequent_itemsets.push(FrequentItemset {
                    items: itemset,
                    support: count as f64 / total_transactions,
                });
            }
        }
    }

    Ok(frequent_itemsets)
}

/// Mine patterns from conditional FP-Tree
fn mine_conditional_tree(
    tree: &FPTree,
    base_pattern: Vec<String>,
    min_support_count: usize,
) -> Vec<(ItemSet, usize)> {
    let mut patterns = Vec::new();

    // Get all items and their counts from the tree
    let item_counts = tree.get_item_counts();

    // Filter by minimum support and sort by frequency
    let mut frequent_items: Vec<(String, usize)> = item_counts
        .into_iter()
        .filter(|(_, count)| *count >= min_support_count)
        .collect();

    frequent_items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    for (item, count) in &frequent_items {
        // Create new pattern by adding this item to base
        let mut new_pattern = base_pattern.clone();
        new_pattern.push(item.clone());
        new_pattern.sort(); // Ensure canonical order

        patterns.push((new_pattern.clone(), *count));

        // Build conditional pattern base for this item
        let cond_patterns = tree.get_conditional_pattern_base(item);

        if !cond_patterns.is_empty() {
            // Build conditional tree
            let mut cond_tree = FPTree::new();
            for (pattern, pattern_count) in &cond_patterns {
                for _ in 0..*pattern_count {
                    cond_tree.insert_transaction(pattern);
                }
            }

            // Recursively mine
            let nested_patterns = mine_conditional_tree(&cond_tree, new_pattern, min_support_count);
            patterns.extend(nested_patterns);
        }
    }

    patterns
}

/// FP-Tree node
#[derive(Debug, Clone)]
struct FPNode {
    item: Option<String>,
    count: usize,
    children: HashMap<String, FPNode>,
}

impl FPNode {
    fn new(item: Option<String>) -> Self {
        Self {
            item,
            count: 0,
            children: HashMap::new(),
        }
    }
}

/// FP-Tree structure
#[derive(Debug)]
struct FPTree {
    root: FPNode,
}

impl FPTree {
    fn new() -> Self {
        Self {
            root: FPNode::new(None),
        }
    }

    /// Insert a transaction into the FP-Tree
    fn insert_transaction(&mut self, items: &[String]) {
        let mut current = &mut self.root;

        for item in items {
            let exists = current.children.contains_key(item);

            if exists {
                current.children.get_mut(item).unwrap().count += 1;
            } else {
                let mut new_node = FPNode::new(Some(item.clone()));
                new_node.count = 1;
                current.children.insert(item.clone(), new_node);
            }

            current = current.children.get_mut(item).unwrap();
        }
    }

    /// Get conditional pattern base for an item
    /// Returns list of (prefix_path, count) tuples
    fn get_conditional_pattern_base(&self, item: &str) -> Vec<(Vec<String>, usize)> {
        let mut patterns = Vec::new();
        let mut current_path = Vec::new();

        // Recursively find all paths ending with the target item
        Self::collect_paths_for_item(&self.root, item, &mut current_path, &mut patterns);

        patterns
    }

    /// Recursively collect all paths ending with target item
    fn collect_paths_for_item(
        node: &FPNode,
        target_item: &str,
        current_path: &mut Vec<String>,
        patterns: &mut Vec<(Vec<String>, usize)>,
    ) {
        // Check if any child matches target item
        if let Some(child) = node.children.get(target_item) {
            // Found a match - add the current path (excluding target) with count
            if !current_path.is_empty() {
                patterns.push((current_path.clone(), child.count));
            }
        }

        // Recursively search all children
        for (item, child) in &node.children {
            current_path.push(item.clone());
            Self::collect_paths_for_item(child, target_item, current_path, patterns);
            current_path.pop();
        }
    }

    /// Get item counts from the tree
    fn get_item_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        Self::count_items_recursive(&self.root, &mut counts);
        counts
    }

    /// Recursively count items in the tree
    fn count_items_recursive(node: &FPNode, counts: &mut HashMap<String, usize>) {
        if let Some(item) = &node.item {
            *counts.entry(item.clone()).or_insert(0) += node.count;
        }

        for child in node.children.values() {
            Self::count_items_recursive(child, counts);
        }
    }
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
    fn test_fpgrowth() {
        let transactions = create_test_transactions();
        let frequent = find_frequent_itemsets(&transactions, 0.5).unwrap();

        // Should find: A, B, C (individual items with >= 50% support)
        assert!(frequent.iter().any(|f| f.items == vec!["A".to_string()]));
        assert!(frequent.iter().any(|f| f.items == vec!["B".to_string()]));
        assert!(frequent.iter().any(|f| f.items == vec!["C".to_string()]));
    }

    #[test]
    fn test_fpgrowth_high_support() {
        let transactions = create_test_transactions();
        let frequent = find_frequent_itemsets(&transactions, 0.75).unwrap();

        // Only A, B, C have >= 75% support (3 out of 4)
        assert!(frequent.iter().any(|f| f.items == vec!["A".to_string()]));
        assert!(frequent.iter().any(|f| f.items == vec!["B".to_string()]));
        assert!(frequent.iter().any(|f| f.items == vec!["C".to_string()]));
    }
}
