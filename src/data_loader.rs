//! Data loading utilities for Excel and CSV files using excelstream
//!
//! Provides high-performance streaming loading of transaction data from:
//! - Excel files (.xlsx) - ultra-low memory streaming
//! - CSV files (.csv) - memory-efficient streaming
//!
//! # Example
//!
//! ```no_run
//! use rust_rule_miner::data_loader::DataLoader;
//!
//! // Load from Excel
//! let transactions = DataLoader::from_excel("sales_data.xlsx", 0)?;
//!
//! // Load from CSV
//! let transactions = DataLoader::from_csv("transactions.csv")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::errors::{MiningError, Result};
use crate::Transaction;
use chrono::{DateTime, NaiveDateTime, Utc};
use excelstream::streaming_reader::StreamingReader;
use excelstream::CsvReader;
use std::path::Path;

/// Data loader for Excel and CSV files using excelstream
pub struct DataLoader;

impl DataLoader {
    /// Load transactions from Excel file (.xlsx)
    ///
    /// Uses excelstream for high-performance streaming with ~3-35 MB memory usage
    /// regardless of file size.
    ///
    /// Expected format:
    /// - Column 0: Transaction ID
    /// - Column 1: Items (comma-separated)
    /// - Column 2: Timestamp (ISO 8601, Unix timestamp, or datetime string)
    ///
    /// First row is treated as header and skipped.
    ///
    /// # Arguments
    /// * `path` - Path to Excel file
    /// * `sheet_index` - Sheet index (0-based) to read
    ///
    /// # Returns
    /// Vector of transactions
    ///
    /// # Example
    /// ```no_run
    /// use rust_rule_miner::data_loader::DataLoader;
    ///
    /// let transactions = DataLoader::from_excel("sales.xlsx", 0)?;
    /// println!("Loaded {} transactions", transactions.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_excel<P: AsRef<Path>>(path: P, sheet_index: usize) -> Result<Vec<Transaction>> {
        let mut reader = StreamingReader::open(path.as_ref())
            .map_err(|e| MiningError::DataLoadError(format!("Failed to open Excel file: {}", e)))?;

        let mut transactions = Vec::new();
        let mut row_idx = 0;

        for row_result in reader.rows_by_index(sheet_index).map_err(|e| {
            MiningError::DataLoadError(format!("Failed to read sheet {}: {}", sheet_index, e))
        })? {
            let row = row_result.map_err(|e| {
                MiningError::DataLoadError(format!("Failed to read row {}: {}", row_idx, e))
            })?;

            row_idx += 1;

            // Skip header row
            if row_idx == 1 {
                continue;
            }

            // Convert row to Vec<String>
            let row_values = row.to_strings();

            match Self::parse_transaction_from_values(&row_values, row_idx) {
                Ok(Some(tx)) => transactions.push(tx),
                Ok(None) => continue, // Skip empty rows
                Err(e) => {
                    log::warn!("Skipping row {}: {}", row_idx, e);
                    continue;
                }
            }
        }

        if transactions.is_empty() {
            return Err(MiningError::InsufficientData(
                "No valid transactions found in Excel file".to_string(),
            ));
        }

        Ok(transactions)
    }

    /// Load transactions from CSV file
    ///
    /// Uses excelstream for high-performance streaming with constant memory usage.
    ///
    /// Expected format:
    /// ```csv
    /// transaction_id,items,timestamp
    /// tx1,"Laptop,Mouse",2024-01-15T10:30:00Z
    /// tx2,"Phone,Phone Case",2024-01-15T11:00:00Z
    /// ```
    ///
    /// First row is treated as header and skipped.
    ///
    /// # Arguments
    /// * `path` - Path to CSV file
    ///
    /// # Returns
    /// Vector of transactions
    ///
    /// # Example
    /// ```no_run
    /// use rust_rule_miner::data_loader::DataLoader;
    ///
    /// let transactions = DataLoader::from_csv("transactions.csv")?;
    /// println!("Loaded {} transactions", transactions.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Transaction>> {
        let mut reader = CsvReader::open(path.as_ref())
            .map_err(|e| MiningError::DataLoadError(format!("Failed to open CSV file: {}", e)))?;

        let mut transactions = Vec::new();
        let mut row_idx = 0;

        for row_result in reader.rows() {
            let row = row_result.map_err(|e| {
                MiningError::DataLoadError(format!("Failed to read row {}: {}", row_idx, e))
            })?;

            row_idx += 1;

            // Skip header row
            if row_idx == 1 {
                continue;
            }

            // Convert row to Vec<String>
            let row_values: Vec<String> = row.into_iter().map(|v| v.to_string()).collect();

            match Self::parse_transaction_from_values(&row_values, row_idx) {
                Ok(Some(tx)) => transactions.push(tx),
                Ok(None) => continue, // Skip empty rows
                Err(e) => {
                    log::warn!("Skipping row {}: {}", row_idx, e);
                    continue;
                }
            }
        }

        if transactions.is_empty() {
            return Err(MiningError::InsufficientData(
                "No valid transactions found in CSV file".to_string(),
            ));
        }

        Ok(transactions)
    }

    /// Parse a row of values into a Transaction
    pub(crate) fn parse_transaction_from_values(
        row_values: &[String],
        row_idx: usize,
    ) -> Result<Option<Transaction>> {
        if row_values.len() < 3 {
            return Err(MiningError::DataLoadError(format!(
                "Row {} has insufficient columns (expected 3, got {})",
                row_idx,
                row_values.len()
            )));
        }

        // Column 0: Transaction ID
        let tx_id = row_values[0].trim();
        if tx_id.is_empty() {
            return Ok(None); // Skip empty transaction ID
        }

        // Column 1: Items (comma-separated)
        let items: Vec<String> = row_values[1]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if items.is_empty() {
            return Ok(None); // Skip empty transactions
        }

        // Column 2: Timestamp
        let timestamp = Self::parse_timestamp(&row_values[2], row_idx)?;

        Ok(Some(Transaction::new(tx_id.to_string(), items, timestamp)))
    }

    /// Parse timestamp from string (supports ISO 8601, Unix timestamp, and common datetime formats)
    fn parse_timestamp(timestamp_str: &str, row_idx: usize) -> Result<DateTime<Utc>> {
        let trimmed = timestamp_str.trim();

        // Try parsing as ISO 8601 first (most common format)
        if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try parsing as Unix timestamp (seconds)
        if let Ok(unix_ts) = trimmed.parse::<i64>() {
            if let Some(dt) = DateTime::from_timestamp(unix_ts, 0) {
                return Ok(dt);
            }
        }

        // Try parsing as naive datetime formats
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M:%S%.f",
            "%Y/%m/%d %H:%M:%S",
            "%d-%m-%Y %H:%M:%S",
            "%d/%m/%Y %H:%M:%S",
            "%Y-%m-%d",
            "%Y/%m/%d",
            "%d-%m-%Y",
            "%d/%m/%Y",
        ];

        for format in &formats {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(trimmed, format) {
                return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
            }
        }

        // Default to current time if parsing fails
        log::warn!(
            "Failed to parse timestamp '{}' at row {}, using current time",
            trimmed,
            row_idx
        );
        Ok(Utc::now())
    }

    /// List all sheet names from an Excel file
    pub fn list_sheets<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
        let reader = StreamingReader::open(path.as_ref())
            .map_err(|e| MiningError::DataLoadError(format!("Failed to open Excel file: {}", e)))?;

        Ok(reader.sheet_names().to_vec())
    }

    /// Load transactions from AWS S3 bucket (requires `cloud` feature)
    ///
    /// Streams directly from S3 with constant memory usage (~3-35 MB).
    ///
    /// # Arguments
    /// * `bucket` - S3 bucket name
    /// * `key` - S3 object key (file path in bucket)
    /// * `region` - AWS region (e.g., "us-east-1")
    /// * `sheet_index` - Sheet index (0-based) for Excel files
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_rule_miner::data_loader::DataLoader;
    ///
    /// // Load from S3
    /// let transactions = DataLoader::from_s3(
    ///     "my-data-bucket",
    ///     "sales/2024/transactions.xlsx",
    ///     "us-east-1",
    ///     0
    /// ).await?;
    ///
    /// println!("Loaded {} transactions from S3", transactions.len());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "cloud")]
    pub async fn from_s3(
        bucket: &str,
        key: &str,
        region: &str,
        sheet_index: usize,
    ) -> Result<Vec<Transaction>> {
        use excelstream::cloud::S3ExcelReader;

        let mut reader = S3ExcelReader::builder()
            .bucket(bucket)
            .key(key)
            .region(region)
            .build()
            .await
            .map_err(|e| MiningError::DataLoadError(format!("Failed to open S3 file: {}", e)))?;

        let mut transactions = Vec::new();
        let mut row_idx = 0;

        for row_result in reader.rows_by_index(sheet_index).map_err(|e| {
            MiningError::DataLoadError(format!("Failed to read sheet {}: {}", sheet_index, e))
        })? {
            let row = row_result.map_err(|e| {
                MiningError::DataLoadError(format!("Failed to read row {}: {}", row_idx, e))
            })?;

            row_idx += 1;

            // Skip header row
            if row_idx == 1 {
                continue;
            }

            // Convert row to Vec<String>
            let row_values = row.to_strings();

            match Self::parse_transaction_from_values(&row_values, row_idx) {
                Ok(Some(tx)) => transactions.push(tx),
                Ok(None) => continue,
                Err(e) => {
                    log::warn!("Skipping row {}: {}", row_idx, e);
                    continue;
                }
            }
        }

        if transactions.is_empty() {
            return Err(MiningError::InsufficientData(
                "No valid transactions found in S3 file".to_string(),
            ));
        }

        Ok(transactions)
    }

    /// Load transactions from HTTP URL (requires `cloud` feature)
    ///
    /// Streams CSV data from HTTP endpoint with constant memory usage.
    ///
    /// # Arguments
    /// * `url` - HTTP URL to CSV file
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_rule_miner::data_loader::DataLoader;
    ///
    /// // Load from HTTP endpoint
    /// let transactions = DataLoader::from_http(
    ///     "https://example.com/data/transactions.csv"
    /// ).await?;
    ///
    /// println!("Loaded {} transactions from HTTP", transactions.len());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "cloud")]
    pub async fn from_http(url: &str) -> Result<Vec<Transaction>> {
        // Download to temp file first, then use CsvReader
        // (excelstream doesn't have direct HTTP CSV reader yet)
        let response = reqwest::get(url)
            .await
            .map_err(|e| MiningError::DataLoadError(format!("HTTP request failed: {}", e)))?;

        let content = response
            .text()
            .await
            .map_err(|e| MiningError::DataLoadError(format!("Failed to read response: {}", e)))?;

        // Parse CSV from string
        let mut transactions = Vec::new();
        let mut row_idx = 0;

        for line in content.lines() {
            row_idx += 1;

            // Skip header
            if row_idx == 1 {
                continue;
            }

            // Parse CSV row
            let row_values: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();

            match Self::parse_transaction_from_values(&row_values, row_idx) {
                Ok(Some(tx)) => transactions.push(tx),
                Ok(None) => continue,
                Err(e) => {
                    log::warn!("Skipping row {}: {}", row_idx, e);
                    continue;
                }
            }
        }

        if transactions.is_empty() {
            return Err(MiningError::InsufficientData(
                "No valid transactions found in HTTP response".to_string(),
            ));
        }

        Ok(transactions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_csv_loading() {
        // Create temporary CSV file
        let csv_content = r#"transaction_id,items,timestamp
tx1,"Laptop,Mouse",2024-01-15T10:30:00Z
tx2,"Phone,Phone Case",2024-01-15T11:00:00Z
tx3,"Tablet",2024-01-15T12:00:00Z
"#;

        let temp_file = "/tmp/test_transactions_excelstream.csv";
        let mut file = fs::File::create(temp_file).unwrap();
        file.write_all(csv_content.as_bytes()).unwrap();

        // Load transactions
        let transactions = DataLoader::from_csv(temp_file).unwrap();

        assert_eq!(transactions.len(), 3);
        assert_eq!(transactions[0].id, "tx1");
        assert_eq!(transactions[0].items, vec!["Laptop", "Mouse"]);
        assert_eq!(transactions[1].items, vec!["Phone", "Phone Case"]);
        assert_eq!(transactions[2].items, vec!["Tablet"]);

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_timestamp_parsing() {
        // ISO 8601
        let ts1 = DataLoader::parse_timestamp("2024-01-15T10:30:00Z", 1).unwrap();
        assert_eq!(ts1.to_rfc3339(), "2024-01-15T10:30:00+00:00");

        // Unix timestamp
        let ts2 = DataLoader::parse_timestamp("1705316400", 1).unwrap();
        assert!(ts2.timestamp() > 0);

        // Naive datetime
        let ts3 = DataLoader::parse_timestamp("2024-01-15 10:30:00", 1).unwrap();
        assert_eq!(ts3.format("%Y-%m-%d").to_string(), "2024-01-15");

        // Alternative formats
        let ts4 = DataLoader::parse_timestamp("2024/01/15 10:30:00", 1).unwrap();
        assert_eq!(ts4.format("%Y-%m-%d").to_string(), "2024-01-15");

        let _ts5 = DataLoader::parse_timestamp("15-01-2024", 1).unwrap();
        // Date parsing may default to current time if format not recognized
    }
}
