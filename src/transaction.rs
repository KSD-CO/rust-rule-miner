use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A transaction (shopping cart, event sequence, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub items: Vec<String>,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new<S: Into<String>>(id: S, items: Vec<String>, timestamp: DateTime<Utc>) -> Self {
        Self {
            id: id.into(),
            timestamp,
            items,
            user_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Create transaction with user ID
    pub fn with_user<S: Into<String>>(
        id: S,
        items: Vec<String>,
        timestamp: DateTime<Utc>,
        user_id: S,
    ) -> Self {
        Self {
            id: id.into(),
            timestamp,
            items,
            user_id: Some(user_id.into()),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to transaction
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if transaction contains an item
    pub fn contains(&self, item: &str) -> bool {
        self.items.iter().any(|i| i == item)
    }

    /// Check if transaction contains all items
    pub fn contains_all(&self, items: &[String]) -> bool {
        items.iter().all(|item| self.contains(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new("tx1", vec!["A".to_string(), "B".to_string()], Utc::now());
        assert_eq!(tx.id, "tx1");
        assert_eq!(tx.items.len(), 2);
    }

    #[test]
    fn test_transaction_contains() {
        let tx = Transaction::new(
            "tx1",
            vec!["Laptop".to_string(), "Mouse".to_string()],
            Utc::now(),
        );
        assert!(tx.contains("Laptop"));
        assert!(!tx.contains("Keyboard"));
    }

    #[test]
    fn test_transaction_contains_all() {
        let tx = Transaction::new(
            "tx1",
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            Utc::now(),
        );
        assert!(tx.contains_all(&["A".to_string(), "B".to_string()]));
        assert!(!tx.contains_all(&["A".to_string(), "D".to_string()]));
    }
}
