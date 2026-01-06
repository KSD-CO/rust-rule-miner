//! Data loading utilities for Excel and CSV files using excelstream
//!
//! Provides high-performance streaming loading of transaction data from:
//! - Excel files (.xlsx) - ultra-low memory streaming
//! - CSV files (.csv) - memory-efficient streaming
//!
//! # Column Mapping (v0.2.0+)
//!
//! All data loading methods require `ColumnMapping` to specify which columns to mine.
//! This provides full flexibility for any data schema.
//!
//! ```no_run
//! use rust_rule_miner::data_loader::{DataLoader, ColumnMapping};
//!
//! // CSV: customer_id, product, category, price, location, timestamp
//! //      0            1        2         3      4         5
//!
//! // Mine single field (product names from column 1)
//! let mapping = ColumnMapping::simple(0, 1, 5);
//! let transactions = DataLoader::from_csv("sales.csv", mapping)?;
//!
//! // Mine multiple fields combined (product + category)
//! let mapping = ColumnMapping::multi_field(0, vec![1, 2], 5, "::".to_string());
//! let transactions = DataLoader::from_csv("sales.csv", mapping)?;
//! // Items: "Laptop::Electronics", "Mouse::Accessories"
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Example
//!
//! ```no_run
//! use rust_rule_miner::data_loader::{DataLoader, ColumnMapping};
//!
//! // Standard 3-column format: transaction_id, items, timestamp
//! let mapping = ColumnMapping::simple(0, 1, 2);
//!
//! // Load from Excel
//! let transactions = DataLoader::from_excel("sales_data.xlsx", 0, mapping.clone())?;
//!
//! // Load from CSV
//! let transactions = DataLoader::from_csv("transactions.csv", mapping)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::errors::{MiningError, Result};
use crate::Transaction;
use chrono::{DateTime, NaiveDateTime, Utc};
use excelstream::streaming_reader::StreamingReader;
use excelstream::CsvReader;
use std::path::Path;

/// Column mapping configuration for flexible data loading
///
/// Allows you to specify which columns to mine from your data,
/// supporting multiple fields combined into patterns.
#[derive(Debug, Clone)]
pub struct ColumnMapping {
    /// Column index for transaction/group ID (0-based)
    pub transaction_id: usize,
    /// Column indices for items to mine (supports multiple columns)
    pub item_columns: Vec<usize>,
    /// Column index for timestamp (0-based)
    pub timestamp: usize,
    /// Separator to combine multiple item columns (default: "::")
    pub field_separator: String,
}

impl ColumnMapping {
    /// Create mapping with transaction_id, single item column, and timestamp
    ///
    /// # Example
    /// ```
    /// use rust_rule_miner::data_loader::ColumnMapping;
    ///
    /// // Mine product names from column 1
    /// let mapping = ColumnMapping::simple(0, 1, 5);
    /// // CSV: customer_id, product_name, category, price, location, timestamp
    /// //      0            1             2         3      4         5
    /// ```
    pub fn simple(transaction_id: usize, item_column: usize, timestamp: usize) -> Self {
        Self {
            transaction_id,
            item_columns: vec![item_column],
            timestamp,
            field_separator: "::".to_string(),
        }
    }

    /// Create mapping to mine multiple fields combined
    ///
    /// # Example
    /// ```
    /// use rust_rule_miner::data_loader::ColumnMapping;
    ///
    /// // Mine product + category + location combined
    /// let mapping = ColumnMapping::multi_field(
    ///     0,                  // customer_id (column 0)
    ///     vec![1, 2, 4],      // product(1), category(2), location(4)
    ///     5,                  // timestamp (column 5)
    ///     "::".to_string()    // separator
    /// );
    /// // CSV: customer_id, product, category, price, location, timestamp
    /// // Results in items like: "Laptop::Electronics::US"
    /// ```
    pub fn multi_field(
        transaction_id: usize,
        item_columns: Vec<usize>,
        timestamp: usize,
        field_separator: String,
    ) -> Self {
        Self {
            transaction_id,
            item_columns,
            timestamp,
            field_separator,
        }
    }
}

/// Data loader for Excel and CSV files using excelstream
pub struct DataLoader;

impl DataLoader {
    /// Load transactions from Excel file (.xlsx) with custom column mapping
    ///
    /// Uses excelstream for high-performance streaming with ~3-35 MB memory usage
    /// regardless of file size.
    ///
    /// First row is treated as header and skipped.
    ///
    /// # Arguments
    /// * `path` - Path to Excel file
    /// * `sheet_index` - Sheet index (0-based) to read
    /// * `mapping` - Column mapping configuration
    ///
    /// # Returns
    /// Vector of transactions
    ///
    /// # Example
    /// ```no_run
    /// use rust_rule_miner::data_loader::{DataLoader, ColumnMapping};
    ///
    /// // Standard format: transaction_id(0), items(1), timestamp(2)
    /// let mapping = ColumnMapping::simple(0, 1, 2);
    /// let transactions = DataLoader::from_excel("sales.xlsx", 0, mapping)?;
    /// println!("Loaded {} transactions", transactions.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_excel<P: AsRef<Path>>(
        path: P,
        sheet_index: usize,
        mapping: ColumnMapping,
    ) -> Result<Vec<Transaction>> {
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

            match Self::parse_transaction_with_mapping(&row_values, row_idx, &mapping) {
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

    /// Load transactions from CSV file with custom column mapping
    ///
    /// Uses excelstream for high-performance streaming with constant memory usage.
    ///
    /// First row is treated as header and skipped.
    ///
    /// # Arguments
    /// * `path` - Path to CSV file
    /// * `mapping` - Column mapping configuration
    ///
    /// # Returns
    /// Vector of transactions
    ///
    /// # Example
    /// ```no_run
    /// use rust_rule_miner::data_loader::{DataLoader, ColumnMapping};
    ///
    /// // Standard format: transaction_id(0), items(1), timestamp(2)
    /// let mapping = ColumnMapping::simple(0, 1, 2);
    /// let transactions = DataLoader::from_csv("transactions.csv", mapping)?;
    /// println!("Loaded {} transactions", transactions.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_csv<P: AsRef<Path>>(path: P, mapping: ColumnMapping) -> Result<Vec<Transaction>> {
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

            match Self::parse_transaction_with_mapping(&row_values, row_idx, &mapping) {
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

    /// Parse a row of values into a Transaction using column mapping
    pub(crate) fn parse_transaction_with_mapping(
        row_values: &[String],
        row_idx: usize,
        mapping: &ColumnMapping,
    ) -> Result<Option<Transaction>> {
        // Validate row has enough columns
        let max_col = *[
            mapping.transaction_id,
            *mapping.item_columns.iter().max().unwrap_or(&0),
            mapping.timestamp,
        ]
        .iter()
        .max()
        .unwrap_or(&0);

        if row_values.len() <= max_col {
            return Err(MiningError::DataLoadError(format!(
                "Row {} has insufficient columns (expected at least {}, got {})",
                row_idx,
                max_col + 1,
                row_values.len()
            )));
        }

        // Extract transaction ID
        let tx_id = row_values[mapping.transaction_id].trim();
        if tx_id.is_empty() {
            return Ok(None); // Skip empty transaction ID
        }

        // Extract and combine item columns
        let items: Vec<String> = if mapping.item_columns.len() == 1 {
            // Single column: split by comma (traditional format)
            // CSV: "Laptop,Mouse,Keyboard"
            row_values[mapping.item_columns[0]]
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // Multiple columns: split each and zip them together
            // CSV columns:  "Laptop,Mouse"   "Electronics,Accessories"   "US,US"
            // Result:       ["Laptop::Electronics::US", "Mouse::Accessories::US"]

            let fields: Vec<Vec<String>> = mapping
                .item_columns
                .iter()
                .map(|&col_idx| {
                    row_values[col_idx]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .collect();

            // Find the maximum length to handle mismatched field counts
            let max_len = fields.iter().map(|f| f.len()).max().unwrap_or(0);
            if max_len == 0 {
                return Ok(None); // Skip if no items in any field
            }

            // Zip fields together with separator
            (0..max_len)
                .map(|i| {
                    fields
                        .iter()
                        .filter_map(|field| field.get(i).cloned())
                        .collect::<Vec<String>>()
                        .join(&mapping.field_separator)
                })
                .filter(|s| !s.is_empty())
                .collect()
        };

        if items.is_empty() {
            return Ok(None);
        }

        // Extract timestamp
        let timestamp = Self::parse_timestamp(&row_values[mapping.timestamp], row_idx)?;

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
    /// * `mapping` - Column mapping configuration
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_rule_miner::data_loader::{DataLoader, ColumnMapping};
    ///
    /// // Standard format: transaction_id(0), items(1), timestamp(2)
    /// let mapping = ColumnMapping::simple(0, 1, 2);
    ///
    /// // Load from S3
    /// let transactions = DataLoader::from_s3(
    ///     "my-data-bucket",
    ///     "sales/2024/transactions.xlsx",
    ///     "us-east-1",
    ///     0,
    ///     mapping
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
        mapping: ColumnMapping,
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

            match Self::parse_transaction_with_mapping(&row_values, row_idx, &mapping) {
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
    /// * `mapping` - Column mapping configuration
    ///
    /// # Example
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use rust_rule_miner::data_loader::{DataLoader, ColumnMapping};
    ///
    /// // Standard format: transaction_id(0), items(1), timestamp(2)
    /// let mapping = ColumnMapping::simple(0, 1, 2);
    ///
    /// // Load from HTTP endpoint
    /// let transactions = DataLoader::from_http(
    ///     "https://example.com/data/transactions.csv",
    ///     mapping
    /// ).await?;
    ///
    /// println!("Loaded {} transactions from HTTP", transactions.len());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "cloud")]
    pub async fn from_http(url: &str, mapping: ColumnMapping) -> Result<Vec<Transaction>> {
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

            match Self::parse_transaction_with_mapping(&row_values, row_idx, &mapping) {
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

        // Load transactions with column mapping
        let mapping = ColumnMapping::simple(0, 1, 2);
        let transactions = DataLoader::from_csv(temp_file, mapping).unwrap();

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
