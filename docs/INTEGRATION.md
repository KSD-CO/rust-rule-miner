# Integration Guide 🔗

Complete guide for integrating rust-rule-miner with rust-rule-engine and other systems.

---

## Table of Contents

1. [rust-rule-engine Integration](#rust-rule-engine-integration)
2. [Web Framework Integration](#web-framework-integration)
3. [Database Integration](#database-integration)
4. [Real-Time Systems](#real-time-systems)
5. [CI/CD Integration](#cicd-integration)
6. [Production Deployment](#production-deployment)

---

## rust-rule-engine Integration

### Complete E-commerce Recommendation System (Simple API - v0.2.0+)

**Using the new `MiningRuleEngine` wrapper for simple integration:**

```rust
use rust_rule_miner::{
    RuleMiner, MiningConfig,
    data_loader::DataLoader,
    engine::{MiningRuleEngine, facts_from_cart},
};

/// Complete workflow: Historical Data → Rule Mining → Engine → Recommendations
pub struct RecommendationSystem {
    engine: MiningRuleEngine,
    rules: Vec<rust_rule_miner::types::AssociationRule>,
}

impl RecommendationSystem {
    /// Initialize from historical sales data
    pub fn from_csv(csv_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // STEP 1: Load historical transaction data
        println!("Loading historical data from {}...", csv_path);
        let transactions = DataLoader::from_csv(csv_path)?;
        println!("✓ Loaded {} transactions", transactions.len());

        // STEP 2: Mine association rules
        println!("Mining association rules...");
        let config = MiningConfig {
            min_support: 0.05,
            min_confidence: 0.60,
            min_lift: 1.5,
            ..Default::default()
        };

        let mut miner = RuleMiner::new(config);
        miner.add_transactions(transactions)?;
        let rules = miner.mine_association_rules()?;
        println!("✓ Mined {} rules", rules.len());

        // STEP 3: Load into engine (automatic GRL conversion!)
        println!("Loading rules into engine...");
        let mut engine = MiningRuleEngine::new("ProductRecommendations");
        engine.load_rules(&rules)?;
        println!("✓ Engine ready with {} rules", rules.len());

        Ok(Self { engine, rules })
    }

    /// Get recommendations for a shopping cart
    pub fn recommend(&mut self, cart_items: Vec<String>) -> Vec<String> {
        // Create facts and execute
        let facts = facts_from_cart(cart_items);
        let result = self.engine.execute(&facts).unwrap();

        // Extract recommendations
        if let Some(rust_rule_engine::Value::Array(recs)) = result.get("Recommendation.items") {
            recs.iter()
                .filter_map(|v| match v {
                    rust_rule_engine::Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Get rule statistics
    pub fn stats(&self) -> RuleStats {
        RuleStats {
            total_rules: self.rules.len(),
            avg_confidence: self.rules.iter()
                .map(|r| r.metrics.confidence)
                .sum::<f64>() / self.rules.len() as f64,
            avg_lift: self.rules.iter()
                .map(|r| r.metrics.lift)
                .sum::<f64>() / self.rules.len() as f64,
        }
    }
}

pub struct RuleStats {
    pub total_rules: usize,
    pub avg_confidence: f64,
    pub avg_lift: f64,
}

// Usage
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut system = RecommendationSystem::from_csv("sales_history.csv")?;

    let recommendations = system.recommend(vec![
        "Laptop".to_string(),
        "Monitor".to_string(),
    ]);

    println!("Recommendations: {:?}", recommendations);

    let stats = system.stats();
    println!("System Stats:");
    println!("  Total Rules: {}", stats.total_rules);
    println!("  Avg Confidence: {:.1}%", stats.avg_confidence * 100.0);
    println!("  Avg Lift: {:.2}", stats.avg_lift);

    Ok(())
}
```

### Advanced: RETE Engine for High Performance

**For systems with many rules (>100), use the RETE engine directly:**

```rust
use rust_rule_miner::{RuleMiner, MiningConfig, data_loader::DataLoader};
use rust_rule_miner::export::{GrlExporter, GrlConfig};
use rust_rule_engine::rete::{IncrementalEngine, TypedFacts, FactValue};
use rust_rule_engine::rete::grl_loader::GrlReteLoader;

pub struct HighPerformanceRecommendationSystem {
    engine: IncrementalEngine,
    rules: Vec<rust_rule_miner::types::AssociationRule>,
}

impl HighPerformanceRecommendationSystem {
    pub fn from_csv(csv_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Load and mine
        let transactions = DataLoader::from_csv(csv_path)?;
        let mut miner = RuleMiner::new(MiningConfig::default());
        miner.add_transactions(transactions)?;
        let rules = miner.mine_association_rules()?;

        // Generate GRL with custom config
        let grl_config = GrlConfig::custom("Cart.products", "Recommendations.items");
        let grl = GrlExporter::to_grl_with_config(&rules, &grl_config);

        // Load into RETE engine
        let mut engine = IncrementalEngine::new();
        GrlReteLoader::load_from_string(&grl, &mut engine)?;

        Ok(Self { engine, rules })
    }

    pub fn recommend(&mut self, cart_items: Vec<String>) -> Vec<String> {
        let mut facts = TypedFacts::new();
        facts.set("Cart.products", FactValue::Array(
            cart_items.iter()
                .map(|s| FactValue::String(s.clone()))
                .collect()
        ));
        facts.set("Recommendations.items", FactValue::Array(vec![]));

        self.engine.insert_typed_facts("Cart", facts.clone());
        self.engine.fire_all(&mut facts, 10);

        if let Some(FactValue::Array(recs)) = facts.get("Recommendations.items") {
            recs.iter()
                .filter_map(|v| match v {
                    FactValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect()
        } else {
            vec![]
        }
    }
}
```

**See also:**
- [examples/integration_with_engine.rs](../examples/integration_with_engine.rs) - Simple engine integration
- [examples/integration_with_rete.rs](../examples/integration_with_rete.rs) - RETE engine for performance
- [examples/flexible_domain_mining.rs](../examples/flexible_domain_mining.rs) - Multi-domain examples

---

## Web Framework Integration

### Actix-Web REST API

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Deserialize)]
struct RecommendRequest {
    cart_items: Vec<String>,
}

#[derive(Serialize)]
struct RecommendResponse {
    recommendations: Vec<String>,
    confidence_scores: Vec<f64>,
}

struct AppState {
    recommendation_system: Arc<RwLock<RecommendationSystem>>,
}

async fn get_recommendations(
    req: web::Json<RecommendRequest>,
    data: web::Data<AppState>,
) -> Result<HttpResponse> {
    let mut system = data.recommendation_system.write().unwrap();
    let recommendations = system.recommend(req.cart_items.clone());
    
    // Get confidence scores from mined rules
    let scores = recommendations.iter()
        .filter_map(|rec| {
            system.rules.iter()
                .find(|r| r.consequent.contains(rec))
                .map(|r| r.metrics.confidence)
        })
        .collect();
    
    Ok(HttpResponse::Ok().json(RecommendResponse {
        recommendations,
        confidence_scores: scores,
    }))
}

async fn retrain_model(data: web::Data<AppState>) -> Result<HttpResponse> {
    let new_system = RecommendationSystem::from_csv("updated_sales.csv")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    *data.recommendation_system.write().unwrap() = new_system;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Model retrained successfully"
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let system = RecommendationSystem::from_csv("sales_history.csv")
        .expect("Failed to initialize recommendation system");
    
    let app_state = web::Data::new(AppState {
        recommendation_system: Arc::new(RwLock::new(system)),
    });
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/recommend", web::post().to(get_recommendations))
            .route("/retrain", web::post().to(retrain_model))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### Axum REST API

```rust
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router, Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

type SharedState = Arc<RwLock<RecommendationSystem>>;

#[derive(Deserialize)]
struct CartRequest {
    items: Vec<String>,
}

#[derive(Serialize)]
struct RecommendationResponse {
    items: Vec<String>,
    count: usize,
}

async fn recommend_handler(
    Extension(state): Extension<SharedState>,
    Json(req): Json<CartRequest>,
) -> Result<Json<RecommendationResponse>, StatusCode> {
    let mut system = state.write().await;
    let items = system.recommend(req.items);
    
    Ok(Json(RecommendationResponse {
        count: items.len(),
        items,
    }))
}

#[tokio::main]
async fn main() {
    let system = RecommendationSystem::from_csv("sales.csv")
        .expect("Failed to load system");
    
    let shared_state = Arc::new(RwLock::new(system));
    
    let app = Router::new()
        .route("/recommend", post(recommend_handler))
        .layer(Extension(shared_state));
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

---

## Database Integration

### PostgreSQL Integration

```rust
use sqlx::{postgres::PgPool, Row};
use rust_rule_miner::Transaction;
use chrono::{DateTime, Utc};

async fn load_transactions_from_db(pool: &PgPool) -> Result<Vec<Transaction>, sqlx::Error> {
    let rows = sqlx::query(r#"
        SELECT 
            transaction_id,
            array_agg(product_name) as items,
            transaction_date
        FROM sales
        WHERE transaction_date >= NOW() - INTERVAL '90 days'
        GROUP BY transaction_id, transaction_date
        ORDER BY transaction_date
    "#)
    .fetch_all(pool)
    .await?;
    
    let transactions: Vec<Transaction> = rows.iter()
        .map(|row| {
            let id: String = row.get("transaction_id");
            let items: Vec<String> = row.get("items");
            let timestamp: DateTime<Utc> = row.get("transaction_date");
            
            Transaction::new(id, items, timestamp)
        })
        .collect();
    
    Ok(transactions)
}

async fn save_rules_to_db(
    pool: &PgPool,
    rules: &[rust_rule_miner::types::AssociationRule]
) -> Result<(), sqlx::Error> {
    for rule in rules {
        sqlx::query(r#"
            INSERT INTO mined_rules (antecedent, consequent, confidence, support, lift)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (antecedent, consequent) 
            DO UPDATE SET 
                confidence = EXCLUDED.confidence,
                support = EXCLUDED.support,
                lift = EXCLUDED.lift,
                updated_at = NOW()
        "#)
        .bind(&rule.antecedent)
        .bind(&rule.consequent)
        .bind(rule.metrics.confidence)
        .bind(rule.metrics.support)
        .bind(rule.metrics.lift)
        .execute(pool)
        .await?;
    }
    
    Ok(())
}

// Complete workflow
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect("postgresql://localhost/ecommerce").await?;
    
    // Load from DB
    let transactions = load_transactions_from_db(&pool).await?;
    
    // Mine rules
    let mut miner = RuleMiner::new(MiningConfig::default());
    miner.add_transactions(transactions)?;
    let rules = miner.mine_association_rules()?;
    
    // Save back to DB
    save_rules_to_db(&pool, &rules).await?;
    
    println!("Mined and saved {} rules", rules.len());
    
    Ok(())
}
```

### Redis Caching

```rust
use redis::{Client, Commands};
use serde_json;

fn cache_recommendations(
    cart_key: &str,
    recommendations: &[String],
    redis_client: &Client,
) -> Result<(), redis::RedisError> {
    let mut con = redis_client.get_connection()?;
    let json = serde_json::to_string(recommendations).unwrap();
    
    // Cache for 1 hour
    con.set_ex(format!("recommendations:{}", cart_key), json, 3600)?;
    
    Ok(())
}

fn get_cached_recommendations(
    cart_key: &str,
    redis_client: &Client,
) -> Option<Vec<String>> {
    let mut con = redis_client.get_connection().ok()?;
    let json: String = con.get(format!("recommendations:{}", cart_key)).ok()?;
    
    serde_json::from_str(&json).ok()
}
```

---

## Real-Time Systems

### Kafka Stream Processing

```rust
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{StreamConsumer, Consumer};
use rdkafka::message::Message;
use std::time::Duration;

async fn process_transaction_stream(
    mut system: RecommendationSystem,
) -> Result<(), Box<dyn std::error::Error>> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "recommendation_processor")
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .create()?;
    
    consumer.subscribe(&["transactions"])?;
    
    loop {
        match consumer.recv().await {
            Ok(msg) => {
                if let Some(payload) = msg.payload_view::<str>() {
                    let tx: Transaction = serde_json::from_str(payload.unwrap())?;
                    
                    // Get recommendations in real-time
                    let recs = system.recommend(tx.items.clone());
                    
                    // Send to recommendation topic
                    println!("Transaction {}: Recommend {:?}", tx.id, recs);
                }
            }
            Err(e) => eprintln!("Kafka error: {}", e),
        }
    }
}
```

### WebSocket Server

```rust
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

async fn handle_connection(
    stream: tokio::net::TcpStream,
    system: Arc<RwLock<RecommendationSystem>>,
) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut write, mut read) = ws_stream.split();
    
    while let Some(msg) = read.next().await {
        let msg = msg.unwrap();
        
        if let Message::Text(text) = msg {
            let cart: Vec<String> = serde_json::from_str(&text).unwrap();
            
            let mut sys = system.write().unwrap();
            let recs = sys.recommend(cart);
            
            let response = serde_json::to_string(&recs).unwrap();
            write.send(Message::Text(response)).await.unwrap();
        }
    }
}
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/retrain-model.yml
name: Retrain Recommendation Model

on:
  schedule:
    # Run daily at 2 AM
    - cron: '0 2 * * *'
  workflow_dispatch:

jobs:
  retrain:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Download Latest Sales Data
        run: |
          aws s3 cp s3://company-data/sales_history.csv ./data/
      
      - name: Mine Rules
        run: |
          cargo run --release --bin mine_rules -- \
            --input ./data/sales_history.csv \
            --output ./models/rules.grl \
            --min-support 0.05 \
            --min-confidence 0.60
      
      - name: Run Tests
        run: cargo test --release
      
      - name: Deploy to Production
        run: |
          aws s3 cp ./models/rules.grl s3://company-models/production/
          kubectl rollout restart deployment/recommendation-service
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/recommendation-service /usr/local/bin/

EXPOSE 8080

CMD ["recommendation-service"]
```

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  recommendation:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./data:/data
      - ./models:/models
    environment:
      - SALES_DATA_PATH=/data/sales_history.csv
      - MODEL_PATH=/models/rules.grl
      - MIN_SUPPORT=0.05
      - MIN_CONFIDENCE=0.60
    depends_on:
      - postgres
      - redis
  
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: ecommerce
      POSTGRES_USER: app
      POSTGRES_PASSWORD: secret
    volumes:
      - pgdata:/var/lib/postgresql/data
  
  redis:
    image: redis:7
    ports:
      - "6379:6379"

volumes:
  pgdata:
```

---

## Production Deployment

### Monitoring & Metrics

```rust
use prometheus::{Encoder, TextEncoder, Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref RECOMMENDATIONS_TOTAL: Counter = 
        register_counter!("recommendations_total", "Total recommendations made").unwrap();
    
    static ref RECOMMENDATION_DURATION: Histogram =
        register_histogram!("recommendation_duration_seconds", "Recommendation latency").unwrap();
    
    static ref RULES_COUNT: prometheus::IntGauge =
        prometheus::register_int_gauge!("active_rules_count", "Number of active rules").unwrap();
}

impl RecommendationSystem {
    pub fn recommend_with_metrics(&mut self, cart_items: Vec<String>) -> Vec<String> {
        let timer = RECOMMENDATION_DURATION.start_timer();
        
        let recs = self.recommend(cart_items);
        
        RECOMMENDATIONS_TOTAL.inc();
        RULES_COUNT.set(self.rules.len() as i64);
        timer.observe_duration();
        
        recs
    }
    
    pub fn metrics_endpoint() -> String {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}
```

### Health Checks

```rust
#[derive(Serialize)]
struct HealthCheck {
    status: String,
    rules_loaded: usize,
    last_training: String,
    avg_confidence: f64,
}

async fn health_handler(
    Extension(state): Extension<SharedState>,
) -> Json<HealthCheck> {
    let system = state.read().await;
    let stats = system.stats();
    
    Json(HealthCheck {
        status: "healthy".to_string(),
        rules_loaded: stats.total_rules,
        last_training: "2026-01-03T14:00:00Z".to_string(),
        avg_confidence: stats.avg_confidence,
    })
}
```

### Graceful Shutdown

```rust
use tokio::signal;

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    
    println!("Shutting down gracefully...");
}

#[tokio::main]
async fn main() {
    let system = RecommendationSystem::from_csv("sales.csv")
        .expect("Failed to initialize");
    
    let app = /* ... your app ... */;
    
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
```

---

## Next Steps

- 📖 Back to [Getting Started](GETTING_STARTED.md)
- 🚀 Read [Advanced Topics](ADVANCED.md)
- 📊 Explore [Performance Tuning](PERFORMANCE.md)

---

**Production Ready Checklist:**
- ✅ Error handling and retry logic
- ✅ Monitoring and alerting
- ✅ Health checks
- ✅ Graceful shutdown
- ✅ Load testing
- ✅ Database connection pooling
- ✅ Caching strategy
- ✅ CI/CD pipeline
- ✅ Docker containerization
- ✅ Kubernetes deployment

**Questions?**
- GitHub: https://github.com/KSD-CO/rust-rule-miner
- Docs: https://docs.rs/rust-rule-miner
