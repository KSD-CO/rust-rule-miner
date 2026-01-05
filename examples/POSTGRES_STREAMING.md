# PostgreSQL Streaming + Rule Mining Example

This example demonstrates how to integrate **rust-rule-miner** with PostgreSQL to:
1. 📊 Stream transaction data from PostgreSQL
2. ⛏️ Mine association rules from the data
3. 🚀 Execute rules in real-time with rust-rule-engine

## Architecture

```
PostgreSQL Database
       ↓
   Stream Data
       ↓
 rust-rule-miner (Mining)
       ↓
  Association Rules
       ↓
rust-rule-engine (Execution)
       ↓
Real-time Recommendations
```

## Setup

### 1. Install PostgreSQL

```bash
# macOS
brew install postgresql@15

# Ubuntu/Debian
sudo apt-get install postgresql postgresql-contrib

# Fedora/RHEL
sudo dnf install postgresql-server
```

### 2. Create Database

```bash
# Start PostgreSQL service
sudo systemctl start postgresql  # Linux
brew services start postgresql   # macOS

# Create database
createdb rule_mining_demo

# Or use psql
psql -U postgres
CREATE DATABASE rule_mining_demo;
\q
```

### 3. Import Sample Data

```bash
# From project root directory
psql rule_mining_demo < examples/postgres_setup.sql
```

This script creates:
- ✅ 3 tables: `products`, `transactions`, `transaction_items`
- ✅ Sample data with 14 transactions
- ✅ Indexes for query optimization
- ✅ `transaction_summary` view for easy querying

### 4. Configure Database URL

```bash
# Option 1: Export environment variable
export DATABASE_URL="postgresql://username:password@localhost/rule_mining_demo"

# Option 2: Use .env file (if using dotenv)
echo 'DATABASE_URL="postgresql://username:password@localhost/rule_mining_demo"' > .env
```

**Note**: Replace `username` and `password` with your credentials.

For default PostgreSQL setup:
```bash
export DATABASE_URL="postgresql://postgres:postgres@localhost/rule_mining_demo"
```

## Run the Example

```bash
# Compile and run with postgres and engine features
cargo run --example postgres_stream_mining --features "postgres,engine"
```

## Sample Output

```
=== PostgreSQL Streaming + Rule Mining Demo ===

📊 Connecting to PostgreSQL database...
✓ Connected successfully

STEP 1: Streaming transactions from database...

✓ Retrieved 14 transactions
  - TXN001 (3 items): ["Keyboard", "Laptop", "Mouse"]
  - TXN002 (4 items): ["Keyboard", "Laptop", "Mouse", "USB-C Hub"]
  ...

STEP 2: Mining association rules from transactions...

Configuration:
  - Min Support: 0.3
  - Min Confidence: 0.6
  - Min Lift: 1.2

✓ Discovered 8 association rules:
  1. ["Laptop"] => ["Mouse"]
     Support: 57.1%, Confidence: 100.0%, Lift: 1.75
  2. ["Phone"] => ["Phone Case"]
     Support: 42.9%, Confidence: 83.3%, Lift: 2.33
  ...

STEP 3: Loading rules into rust-rule-engine...

✓ Loaded 8 rules into engine

STEP 4: Testing real-time product recommendations...

Test 1: Customer adds Laptop:
  Cart: ["Laptop"]
  Rules fired: 1
  Recommendations: ["Mouse"]
...
```

## Database Structure

### Tables

#### products
```sql
product_id | product_name | category | price
-----------|--------------|----------|-------
1          | Laptop       | Electronics | 999.99
2          | Mouse        | Electronics | 29.99
...
```

#### transactions
```sql
transaction_id | customer_id | transaction_date | total_amount
---------------|-------------|------------------|-------------
TXN001        | CUST001     | 2024-01-15       | 1109.97
...
```

#### transaction_items
```sql
id | transaction_id | product_name | quantity | unit_price
---|----------------|--------------|----------|----------
1  | TXN001        | Laptop       | 1        | 999.99
2  | TXN001        | Mouse        | 1        | 29.99
...
```

## Sample Patterns in Data

The sample data includes these patterns:

### Pattern 1: Laptop Buyers (Support: ~57%)
- Laptop → Mouse (Confidence: 100%)
- Laptop → Keyboard (Confidence: 71%)
- Laptop → Laptop Bag (Confidence: 43%)

### Pattern 2: Phone Buyers (Support: ~43%)
- Phone → Phone Case (Confidence: 83%)
- Phone → Screen Protector (Confidence: 67%)

### Pattern 3: Monitor Buyers (Support: ~21%)
- Monitor → Headphones (Confidence: 100%)
- Monitor → Webcam (Confidence: 67%)

## Tuning Parameters

You can adjust mining parameters in the code:

```rust
let config = MiningConfig {
    min_support: 0.3,      // 30% - minimum transaction frequency
    min_confidence: 0.6,   // 60% - rule reliability
    min_lift: 1.2,         // 1.2 - correlation strength
    ..Default::default()
};
```

### Parameter Explanation:

- **min_support**: Minimum percentage of transactions containing the itemset. Higher value → fewer but more popular rules
- **min_confidence**: Probability of consequent appearing when antecedent is present. Higher value → more accurate rules
- **min_lift**: Measures correlation strength. Lift > 1 indicates positive correlation

## Advanced Usage

### Batch Streaming

```rust
// Stream in chunks for large databases
let mut offset = 0;
let batch_size = 1000;

loop {
    let query = format!(
        "SELECT ... FROM transactions ... LIMIT {} OFFSET {}",
        batch_size, offset
    );
    let rows = client.query(&query, &[]).await?;

    if rows.is_empty() {
        break;
    }

    // Process batch
    // ...

    offset += batch_size;
}
```

### Connection Pooling with bb8

```rust
use bb8_postgres::PostgresConnectionManager;
use bb8::Pool;

let manager = PostgresConnectionManager::new_from_stringlike(
    database_url,
    NoTls,
)?;

let pool = Pool::builder()
    .max_size(15)
    .build(manager)
    .await?;

let conn = pool.get().await?;
// Use connection...
```

### Real-time Notifications with PostgreSQL LISTEN/NOTIFY

```rust
// Listen for new transactions
let mut stream = client.query_raw(
    "LISTEN new_transaction",
    std::iter::empty::<String>(),
).await?;

// React to new transactions
while let Some(notification) = stream.next().await {
    // Re-mine rules with updated data
    // ...
}
```

## Troubleshooting

### Connection refused
```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Start if not running
sudo systemctl start postgresql
```

### Permission denied
```bash
# Create new user if needed
sudo -u postgres createuser --interactive --pwprompt

# Grant permissions
psql -U postgres
GRANT ALL PRIVILEGES ON DATABASE rule_mining_demo TO your_username;
```

### No rules mined
- Check if data was imported: `SELECT COUNT(*) FROM transactions;`
- Lower `min_support` and `min_confidence` to find more rules
- Verify data contains enough patterns

## Next Steps

1. **Production Deployment**: Use connection pooling (bb8)
2. **Real-time Updates**: Implement PostgreSQL LISTEN/NOTIFY for real-time mining
3. **Distributed Processing**: Scale with multiple workers
4. **Caching**: Cache mined rules to avoid re-mining
5. **Monitoring**: Add metrics and logging for production

## References

- [rust-rule-miner Documentation](https://docs.rs/rust-rule-miner)
- [rust-rule-engine Documentation](https://docs.rs/rust-rule-engine)
- [tokio-postgres Documentation](https://docs.rs/tokio-postgres)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
