-- PostgreSQL Setup for rust-rule-miner streaming example
-- This script creates tables and sample data for demonstrating real-time rule mining

-- Drop tables if they exist
DROP TABLE IF EXISTS transaction_items CASCADE;
DROP TABLE IF EXISTS transactions CASCADE;
DROP TABLE IF EXISTS products CASCADE;

-- Products table
CREATE TABLE products (
    product_id SERIAL PRIMARY KEY,
    product_name VARCHAR(100) NOT NULL UNIQUE,
    category VARCHAR(50),
    price DECIMAL(10, 2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Transactions table
CREATE TABLE transactions (
    transaction_id VARCHAR(50) PRIMARY KEY,
    customer_id VARCHAR(50),
    transaction_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    total_amount DECIMAL(10, 2),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Transaction items (many-to-many relationship)
CREATE TABLE transaction_items (
    id SERIAL PRIMARY KEY,
    transaction_id VARCHAR(50) REFERENCES transactions(transaction_id) ON DELETE CASCADE,
    product_name VARCHAR(100) NOT NULL,
    quantity INTEGER DEFAULT 1,
    unit_price DECIMAL(10, 2)
);

-- Create indexes for better query performance
CREATE INDEX idx_transaction_date ON transactions(transaction_date);
CREATE INDEX idx_transaction_items_txid ON transaction_items(transaction_id);
CREATE INDEX idx_customer_id ON transactions(customer_id);

-- Insert sample products
INSERT INTO products (product_name, category, price) VALUES
    ('Laptop', 'Electronics', 999.99),
    ('Mouse', 'Electronics', 29.99),
    ('Keyboard', 'Electronics', 79.99),
    ('USB-C Hub', 'Electronics', 49.99),
    ('Laptop Bag', 'Accessories', 39.99),
    ('Phone', 'Electronics', 799.99),
    ('Phone Case', 'Accessories', 19.99),
    ('Screen Protector', 'Accessories', 9.99),
    ('Wireless Charger', 'Electronics', 39.99),
    ('Headphones', 'Electronics', 149.99),
    ('Webcam', 'Electronics', 89.99),
    ('Monitor', 'Electronics', 299.99),
    ('Desk Lamp', 'Office', 34.99),
    ('Cable Organizer', 'Accessories', 12.99),
    ('External SSD', 'Electronics', 129.99);

-- Insert sample transactions
-- Pattern 1: Laptop buyers often buy Mouse, Keyboard, and accessories
INSERT INTO transactions (transaction_id, customer_id, transaction_date, total_amount) VALUES
    ('TXN001', 'CUST001', '2024-01-15 10:30:00', 1109.97),
    ('TXN002', 'CUST002', '2024-01-15 11:45:00', 1159.96),
    ('TXN003', 'CUST003', '2024-01-15 14:20:00', 1029.98),
    ('TXN004', 'CUST004', '2024-01-16 09:15:00', 1089.96);

INSERT INTO transaction_items (transaction_id, product_name, quantity, unit_price) VALUES
    ('TXN001', 'Laptop', 1, 999.99),
    ('TXN001', 'Mouse', 1, 29.99),
    ('TXN001', 'Keyboard', 1, 79.99),
    ('TXN002', 'Laptop', 1, 999.99),
    ('TXN002', 'Mouse', 1, 29.99),
    ('TXN002', 'USB-C Hub', 1, 49.99),
    ('TXN002', 'Keyboard', 1, 79.99),
    ('TXN003', 'Laptop', 1, 999.99),
    ('TXN003', 'Mouse', 1, 29.99),
    ('TXN004', 'Laptop', 1, 999.99),
    ('TXN004', 'Laptop Bag', 1, 39.99),
    ('TXN004', 'Mouse', 1, 29.99);

-- Pattern 2: Phone buyers often buy Phone Case and Screen Protector
INSERT INTO transactions (transaction_id, customer_id, transaction_date, total_amount) VALUES
    ('TXN005', 'CUST005', '2024-01-16 13:30:00', 819.98),
    ('TXN006', 'CUST006', '2024-01-16 15:45:00', 829.97),
    ('TXN007', 'CUST007', '2024-01-17 10:00:00', 869.96),
    ('TXN008', 'CUST008', '2024-01-17 11:30:00', 809.98);

INSERT INTO transaction_items (transaction_id, product_name, quantity, unit_price) VALUES
    ('TXN005', 'Phone', 1, 799.99),
    ('TXN005', 'Phone Case', 1, 19.99),
    ('TXN006', 'Phone', 1, 799.99),
    ('TXN006', 'Phone Case', 1, 19.99),
    ('TXN006', 'Screen Protector', 1, 9.99),
    ('TXN007', 'Phone', 1, 799.99),
    ('TXN007', 'Phone Case', 1, 19.99),
    ('TXN007', 'Wireless Charger', 1, 39.99),
    ('TXN007', 'Screen Protector', 1, 9.99),
    ('TXN008', 'Phone', 1, 799.99),
    ('TXN008', 'Screen Protector', 1, 9.99);

-- Pattern 3: Monitor buyers often buy Webcam and Headphones (work-from-home setup)
INSERT INTO transactions (transaction_id, customer_id, transaction_date, total_amount) VALUES
    ('TXN009', 'CUST009', '2024-01-17 14:20:00', 539.98),
    ('TXN010', 'CUST010', '2024-01-18 09:30:00', 449.98),
    ('TXN011', 'CUST011', '2024-01-18 11:00:00', 584.96);

INSERT INTO transaction_items (transaction_id, product_name, quantity, unit_price) VALUES
    ('TXN009', 'Monitor', 1, 299.99),
    ('TXN009', 'Webcam', 1, 89.99),
    ('TXN009', 'Headphones', 1, 149.99),
    ('TXN010', 'Monitor', 1, 299.99),
    ('TXN010', 'Headphones', 1, 149.99),
    ('TXN011', 'Monitor', 1, 299.99),
    ('TXN011', 'Webcam', 1, 89.99),
    ('TXN011', 'Desk Lamp', 1, 34.99),
    ('TXN011', 'Headphones', 1, 149.99);

-- More transactions for statistical significance
INSERT INTO transactions (transaction_id, customer_id, transaction_date, total_amount) VALUES
    ('TXN012', 'CUST012', '2024-01-18 15:30:00', 1149.95),
    ('TXN013', 'CUST013', '2024-01-19 10:15:00', 829.97),
    ('TXN014', 'CUST014', '2024-01-19 13:45:00', 1279.94);

INSERT INTO transaction_items (transaction_id, product_name, quantity, unit_price) VALUES
    ('TXN012', 'Laptop', 1, 999.99),
    ('TXN012', 'Mouse', 1, 29.99),
    ('TXN012', 'Keyboard', 1, 79.99),
    ('TXN012', 'Laptop Bag', 1, 39.99),
    ('TXN013', 'Phone', 1, 799.99),
    ('TXN013', 'Phone Case', 1, 19.99),
    ('TXN013', 'Screen Protector', 1, 9.99),
    ('TXN014', 'Laptop', 1, 999.99),
    ('TXN014', 'External SSD', 1, 129.99),
    ('TXN014', 'Mouse', 1, 29.99),
    ('TXN014', 'Keyboard', 1, 79.99),
    ('TXN014', 'Laptop Bag', 1, 39.99);

-- Create a view for easy querying
CREATE OR REPLACE VIEW transaction_summary AS
SELECT
    t.transaction_id,
    t.customer_id,
    t.transaction_date,
    t.total_amount,
    array_agg(ti.product_name ORDER BY ti.product_name) as items,
    count(ti.product_name) as item_count
FROM transactions t
JOIN transaction_items ti ON t.transaction_id = ti.transaction_id
GROUP BY t.transaction_id, t.customer_id, t.transaction_date, t.total_amount
ORDER BY t.transaction_date DESC;

-- Sample query to verify data
-- SELECT * FROM transaction_summary;

COMMIT;

-- Grant permissions (adjust as needed for your setup)
-- GRANT SELECT ON ALL TABLES IN SCHEMA public TO your_user;
