# Graph-Based Rule Mining

**Status**: Planning Phase
**Priority**: Medium-High
**Estimated Impact**: Automatic rule discovery from historical data
**Complexity**: High
**Dependencies**: `petgraph` (already in use for backward-chaining)

---

## 📋 Executive Summary

Automatically discover rules from historical transaction/event data using graph-based pattern mining algorithms. This feature enables the engine to **learn rules** from data instead of requiring manual rule authoring.

**Current Problem:**
- Rules must be manually written by domain experts
- Discovering patterns in large datasets is time-consuming
- Hidden correlations and patterns go unnoticed
- No way to validate if manually-written rules match actual data patterns

**Proposed Solution:**
- Analyze historical data to find frequent patterns
- Use graph representation to model entity relationships
- Generate GRL rules automatically with confidence scores
- Integrate with backward chaining for rule validation/explanation

---

## 🎯 Goals

### Primary Goals
1. **Automatic Pattern Discovery** - Find frequent itemsets, sequential patterns, and correlations
2. **GRL Rule Generation** - Convert discovered patterns into executable GRL rules
3. **Quality Metrics** - Provide confidence, support, and lift scores for each discovered rule
4. **Graph Visualization** - Visualize entity relationships and discovered patterns
5. **Integration** - Work seamlessly with existing Forward/Backward engines

### Non-Goals (Phase 1)
- ❌ Real-time online learning (only batch/offline mining)
- ❌ Deep learning / neural network integration
- ❌ Distributed mining across clusters
- ❌ Streaming data mining (future: combine with Stream Processing)

---

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                   Rule Mining Pipeline                   │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  1. Data Input                                           │
│     ├─ Transactions (JSON/CSV)                          │
│     ├─ Event Logs                                       │
│     └─ Facts Database                                   │
│                          ↓                               │
│  2. Graph Construction                                   │
│     ├─ Build entity graph (petgraph)                    │
│     ├─ Nodes: Products, Users, Events                   │
│     └─ Edges: Relationships (bought_with, followed_by)  │
│                          ↓                               │
│  3. Pattern Mining                                       │
│     ├─ Frequent Itemset Mining (Apriori/FP-Growth)     │
│     ├─ Sequential Pattern Mining                        │
│     ├─ Association Rule Generation                      │
│     └─ Graph Pattern Matching                           │
│                          ↓                               │
│  4. Rule Filtering & Ranking                            │
│     ├─ Calculate: Confidence, Support, Lift            │
│     ├─ Filter low-quality rules                        │
│     └─ Rank by business value                          │
│                          ↓                               │
│  5. GRL Code Generation                                 │
│     ├─ Convert patterns → GRL syntax                    │
│     ├─ Add metadata (salience, confidence)             │
│     └─ Generate explanatory comments                    │
│                          ↓                               │
│  6. Validation (Optional)                               │
│     ├─ Test against validation dataset                 │
│     ├─ Backward chaining verification                   │
│     └─ Human-in-the-loop review                        │
│                                                           │
└─────────────────────────────────────────────────────────┘
```

---

## 🔧 Technical Design

### Core Data Structures

```rust
/// Main rule mining engine
pub struct RuleMiner {
    /// Graph of entity relationships
    entity_graph: Graph<Entity, Relationship>,

    /// Historical transactions/events
    transactions: Vec<Transaction>,

    /// Discovered patterns
    patterns: Vec<Pattern>,

    /// Mining configuration
    config: MiningConfig,

    /// Statistics collector
    stats: MiningStats,
}

/// Entity in the graph (product, user, event, etc.)
#[derive(Debug, Clone)]
pub struct Entity {
    id: String,
    entity_type: EntityType,
    attributes: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Product,
    User,
    Event,
    Category,
    Custom(String),
}

/// Relationship edge in the graph
#[derive(Debug, Clone)]
pub struct Relationship {
    /// Type of relationship
    rel_type: RelationType,

    /// Statistical confidence (0.0 - 1.0)
    confidence: f64,

    /// Support count (number of occurrences)
    support: usize,

    /// Lift (correlation strength)
    lift: f64,

    /// Optional time window
    time_window: Option<Duration>,

    /// Additional metadata
    metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationType {
    /// Items bought together in same transaction
    BoughtTogether,

    /// Items bought in sequence (with time gap)
    BoughtSequentially { max_gap: Duration },

    /// Items that substitute each other (either/or)
    Substitutes,

    /// Items never bought together (negative correlation)
    NeverBoughtTogether,

    /// Custom relationship type
    Custom(String),
}

/// A transaction (shopping cart, event sequence, etc.)
#[derive(Debug, Clone)]
pub struct Transaction {
    id: String,
    timestamp: DateTime<Utc>,
    items: Vec<String>,
    user_id: Option<String>,
    metadata: HashMap<String, Value>,
}

/// Discovered pattern
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Pattern type
    pattern_type: PatternType,

    /// Items in the pattern
    items: Vec<String>,

    /// Quality metrics
    metrics: PatternMetrics,

    /// Discovered from which transactions
    evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    /// Frequent itemset (co-occurrence)
    FrequentItemset,

    /// Sequential pattern (ordered)
    Sequential { time_constraints: Vec<Duration> },

    /// Association rule (A → B)
    AssociationRule { antecedent: Vec<String>, consequent: Vec<String> },

    /// Graph pattern (subgraph match)
    GraphPattern { subgraph: Graph<Entity, Relationship> },
}

/// Pattern quality metrics
#[derive(Debug, Clone)]
pub struct PatternMetrics {
    /// Confidence: P(consequent | antecedent)
    /// How often B happens when A happens
    confidence: f64,

    /// Support: P(antecedent ∧ consequent)
    /// How common the pattern is overall
    support: f64,

    /// Lift: confidence / P(consequent)
    /// Strength of correlation (>1: positive, <1: negative, =1: independent)
    lift: f64,

    /// Conviction: P(A) * P(¬B) / P(A ∧ ¬B)
    /// How much more often A implies B than expected by chance
    conviction: f64,

    /// Optional: time-based metrics
    avg_time_gap: Option<Duration>,
    time_variance: Option<Duration>,
}

/// Mining configuration
#[derive(Debug, Clone)]
pub struct MiningConfig {
    /// Minimum support threshold (0.0 - 1.0)
    /// Example: 0.1 = pattern must appear in at least 10% of transactions
    min_support: f64,

    /// Minimum confidence threshold (0.0 - 1.0)
    /// Example: 0.7 = rule must be correct at least 70% of the time
    min_confidence: f64,

    /// Minimum lift threshold
    /// Example: 1.2 = items must co-occur 20% more than random chance
    min_lift: f64,

    /// Maximum time gap for sequential patterns
    max_time_gap: Option<Duration>,

    /// Mining algorithm to use
    algorithm: MiningAlgorithm,
}

#[derive(Debug, Clone)]
pub enum MiningAlgorithm {
    /// Apriori algorithm (classic, easy to understand)
    Apriori,

    /// FP-Growth (faster, more memory efficient)
    FPGrowth,

    /// Eclat (uses vertical data format)
    Eclat,
}
```

---

## 📐 Implementation Plan

### Stage 1: Data Input & Graph Construction (Week 1-2)

**Tasks:**
1. Implement `Transaction` parsing from JSON/CSV
2. Build entity graph from transactions
3. Calculate basic statistics (item frequencies)
4. Visualize graph using DOT format

**Files:**
- `src/mining/mod.rs` (new module)
- `src/mining/transaction.rs`
- `src/mining/graph_builder.rs`
- `src/mining/stats.rs`

**Example Usage:**
```rust
use rust_rule_engine::mining::RuleMiner;

// Load transactions
let transactions = RuleMiner::load_from_json("transactions.json")?;

// Build graph
let mut miner = RuleMiner::new();
miner.add_transactions(transactions)?;

// Visualize
miner.export_graph_dot("entity_graph.dot")?;
```

**Success Criteria:**
- Parse 10,000+ transactions efficiently (<100ms)
- Build graph with proper entity/relationship modeling
- Export visualization to DOT format

---

### Stage 2: Frequent Itemset Mining (Week 3-4)

**Implementation: Apriori Algorithm**

```rust
impl RuleMiner {
    /// Find all frequent itemsets using Apriori algorithm
    pub fn find_frequent_itemsets(&self) -> Result<Vec<Itemset>> {
        let mut frequent_itemsets = Vec::new();

        // Level 1: Individual items
        let mut current_level = self.generate_1_itemsets();
        let mut k = 1;

        while !current_level.is_empty() {
            println!("Level {}: {} candidates", k, current_level.len());

            // Count support for each candidate
            let counts = self.count_support(&current_level);

            // Filter by minimum support
            let frequent_k = counts.into_iter()
                .filter(|(itemset, support)| {
                    *support >= self.config.min_support
                })
                .collect::<Vec<_>>();

            if frequent_k.is_empty() {
                break;
            }

            frequent_itemsets.extend(frequent_k.clone());

            // Generate next level candidates
            current_level = self.generate_candidates(&frequent_k);
            k += 1;
        }

        Ok(frequent_itemsets)
    }

    /// Generate 1-itemsets (individual items)
    fn generate_1_itemsets(&self) -> Vec<ItemSet> {
        let mut items = HashSet::new();
        for tx in &self.transactions {
            for item in &tx.items {
                items.insert(item.clone());
            }
        }
        items.into_iter().map(|i| vec![i]).collect()
    }

    /// Generate (k+1)-itemsets from k-itemsets
    fn generate_candidates(&self, frequent_k: &[(ItemSet, f64)]) -> Vec<ItemSet> {
        let mut candidates = Vec::new();

        for i in 0..frequent_k.len() {
            for j in (i+1)..frequent_k.len() {
                let (set1, _) = &frequent_k[i];
                let (set2, _) = &frequent_k[j];

                // Join if they share k-1 items
                if self.can_join(set1, set2) {
                    let mut new_set = set1.clone();
                    new_set.push(set2.last().unwrap().clone());
                    new_set.sort();
                    candidates.push(new_set);
                }
            }
        }

        // Prune candidates with infrequent subsets
        candidates.into_iter()
            .filter(|c| self.has_frequent_subsets(c, frequent_k))
            .collect()
    }

    /// Count support for itemsets
    fn count_support(&self, itemsets: &[ItemSet]) -> Vec<(ItemSet, f64)> {
        let total_transactions = self.transactions.len() as f64;

        itemsets.iter()
            .map(|itemset| {
                let count = self.transactions.iter()
                    .filter(|tx| itemset.iter().all(|item| tx.items.contains(item)))
                    .count() as f64;

                let support = count / total_transactions;
                (itemset.clone(), support)
            })
            .collect()
    }
}
```

**Success Criteria:**
- Find all frequent itemsets with configurable min_support
- Handle datasets with 100+ unique items
- Performance: <1 second for 1,000 transactions

---

### Stage 3: Association Rule Generation (Week 5)

```rust
impl RuleMiner {
    /// Generate association rules from frequent itemsets
    pub fn generate_association_rules(
        &self,
        frequent_itemsets: &[(ItemSet, f64)]
    ) -> Vec<AssociationRule> {
        let mut rules = Vec::new();

        for (itemset, support) in frequent_itemsets {
            if itemset.len() < 2 {
                continue; // Need at least 2 items for a rule
            }

            // Generate all possible splits: A → B
            // where A ∪ B = itemset, A ∩ B = ∅
            for antecedent in self.generate_non_empty_subsets(itemset) {
                let consequent: ItemSet = itemset.iter()
                    .filter(|item| !antecedent.contains(item))
                    .cloned()
                    .collect();

                if consequent.is_empty() {
                    continue;
                }

                // Calculate metrics
                let confidence = self.calculate_confidence(&antecedent, &consequent);
                let lift = self.calculate_lift(&antecedent, &consequent, *support);
                let conviction = self.calculate_conviction(&antecedent, &consequent);

                // Filter by thresholds
                if confidence >= self.config.min_confidence
                    && lift >= self.config.min_lift {
                    rules.push(AssociationRule {
                        antecedent: antecedent.clone(),
                        consequent: consequent.clone(),
                        metrics: PatternMetrics {
                            confidence,
                            support: *support,
                            lift,
                            conviction,
                            avg_time_gap: None,
                            time_variance: None,
                        },
                    });
                }
            }
        }

        // Sort by confidence * lift (quality score)
        rules.sort_by(|a, b| {
            let score_a = a.metrics.confidence * a.metrics.lift;
            let score_b = b.metrics.confidence * b.metrics.lift;
            score_b.partial_cmp(&score_a).unwrap()
        });

        rules
    }

    fn calculate_confidence(&self, antecedent: &ItemSet, consequent: &ItemSet) -> f64 {
        // Confidence = P(A ∧ B) / P(A)
        let both_count = self.count_transactions_with_all(antecedent, consequent);
        let antecedent_count = self.count_transactions_with(antecedent);

        if antecedent_count == 0 {
            return 0.0;
        }

        both_count as f64 / antecedent_count as f64
    }

    fn calculate_lift(&self, antecedent: &ItemSet, consequent: &ItemSet, support: f64) -> f64 {
        // Lift = P(A ∧ B) / (P(A) * P(B))
        let consequent_support = self.get_support(consequent);

        if consequent_support == 0.0 {
            return 0.0;
        }

        let confidence = self.calculate_confidence(antecedent, consequent);
        confidence / consequent_support
    }

    fn calculate_conviction(&self, antecedent: &ItemSet, consequent: &ItemSet) -> f64 {
        // Conviction = P(A) * P(¬B) / P(A ∧ ¬B)
        // High conviction means rule is not accidental
        let p_a = self.get_support(antecedent);
        let p_b = self.get_support(consequent);
        let confidence = self.calculate_confidence(antecedent, consequent);

        if confidence >= 1.0 {
            return f64::INFINITY;
        }

        (1.0 - p_b) / (1.0 - confidence)
    }
}
```

**Success Criteria:**
- Generate high-quality rules with confidence/support/lift metrics
- Filter out spurious correlations (low lift)
- Rank rules by business value

---

### Stage 4: Sequential Pattern Mining (Week 6)

```rust
#[derive(Debug, Clone)]
pub struct SequentialPattern {
    /// Sequence of itemsets
    sequence: Vec<ItemSet>,

    /// Time constraints between steps
    time_gaps: Vec<Duration>,

    /// Support across all customer sequences
    support: f64,
}

impl RuleMiner {
    /// Find sequential patterns (items bought in order)
    pub fn find_sequential_patterns(&self) -> Vec<SequentialPattern> {
        // Group transactions by customer
        let customer_sequences = self.group_transactions_by_customer();

        let mut patterns = HashMap::new();

        for (customer_id, transactions) in customer_sequences {
            // Sort by timestamp
            let mut sorted = transactions.clone();
            sorted.sort_by_key(|t| t.timestamp);

            // Find sequences within time window
            for window_size in 2..=sorted.len() {
                for window in sorted.windows(window_size) {
                    let time_gaps: Vec<Duration> = window.windows(2)
                        .map(|pair| {
                            let gap = pair[1].timestamp - pair[0].timestamp;
                            Duration::from_std(gap.to_std().unwrap()).unwrap()
                        })
                        .collect();

                    // Check if all gaps are within max_time_gap
                    if let Some(max_gap) = self.config.max_time_gap {
                        if time_gaps.iter().any(|&gap| gap > max_gap) {
                            continue;
                        }
                    }

                    let sequence: Vec<ItemSet> = window.iter()
                        .map(|tx| tx.items.clone())
                        .collect();

                    let pattern = SequentialPattern {
                        sequence: sequence.clone(),
                        time_gaps: time_gaps.clone(),
                        support: 0.0, // Will calculate later
                    };

                    *patterns.entry(sequence).or_insert(0) += 1;
                }
            }
        }

        // Calculate support
        let total_customers = customer_sequences.len() as f64;
        patterns.into_iter()
            .map(|(seq, count)| SequentialPattern {
                sequence: seq.clone(),
                time_gaps: vec![], // Simplified
                support: count as f64 / total_customers,
            })
            .filter(|p| p.support >= self.config.min_support)
            .collect()
    }
}
```

**Success Criteria:**
- Discover sequential patterns with time constraints
- Example: "Laptop → (2 days) → Mouse → (5 days) → Laptop Bag"
- Support configurable time windows

---

### Stage 5: GRL Code Generation (Week 7)

```rust
impl RuleMiner {
    /// Convert discovered rules to GRL syntax
    pub fn to_grl(&self, rules: &[AssociationRule]) -> String {
        let mut grl = String::new();

        grl.push_str("// Auto-generated rules from pattern mining\n");
        grl.push_str(&format!("// Generated: {}\n", Utc::now()));
        grl.push_str(&format!("// Total rules: {}\n", rules.len()));
        grl.push_str(&format!("// Min confidence: {:.1}%\n", self.config.min_confidence * 100.0));
        grl.push_str(&format!("// Min support: {:.1}%\n", self.config.min_support * 100.0));
        grl.push_str("\n");

        for (idx, rule) in rules.iter().enumerate() {
            let rule_name = self.generate_rule_name(rule, idx);
            let salience = (rule.metrics.confidence * 100.0) as i32;

            grl.push_str(&format!(r#"
// Rule #{}: {} → {}
// Confidence: {:.1}% | Support: {:.1}% | Lift: {:.2} | Conviction: {:.2}
// Interpretation: When {} present, {} appears {:.1}% of the time
rule "{}" {{
    salience {}
    when
        {}
    then
        {};
        LogMessage("Rule fired: {} (confidence: {:.1}%)");
}}
"#,
                idx + 1,
                rule.antecedent.join(", "),
                rule.consequent.join(", "),
                rule.metrics.confidence * 100.0,
                rule.metrics.support * 100.0,
                rule.metrics.lift,
                rule.metrics.conviction,
                rule.antecedent.join(", "),
                rule.consequent.join(", "),
                rule.metrics.confidence * 100.0,
                rule_name,
                salience,
                self.generate_conditions(&rule.antecedent),
                self.generate_actions(&rule.consequent),
                rule_name,
                rule.metrics.confidence * 100.0
            ));
        }

        grl
    }

    fn generate_rule_name(&self, rule: &AssociationRule, idx: usize) -> String {
        let antecedent_str = rule.antecedent.iter()
            .map(|s| s.replace(" ", "_"))
            .collect::<Vec<_>>()
            .join("_");

        let consequent_str = rule.consequent.iter()
            .map(|s| s.replace(" ", "_"))
            .collect::<Vec<_>>()
            .join("_");

        format!("Mined_{}_Implies_{}", antecedent_str, consequent_str)
    }

    fn generate_conditions(&self, items: &ItemSet) -> String {
        items.iter()
            .map(|item| format!("ShoppingCart.contains(\"{}\")", item))
            .collect::<Vec<_>>()
            .join(" &&\n        ")
    }

    fn generate_actions(&self, items: &ItemSet) -> String {
        items.iter()
            .map(|item| format!("Recommendation.add(\"{}\")", item))
            .collect::<Vec<_>>()
            .join(";\n        ")
    }
}
```

**Example Output:**
```grl
// Auto-generated rules from pattern mining
// Generated: 2026-01-02 10:30:00 UTC
// Total rules: 15
// Min confidence: 70.0%
// Min support: 10.0%

// Rule #1: Laptop → Mouse
// Confidence: 85.7% | Support: 60.0% | Lift: 1.43 | Conviction: 2.33
// Interpretation: When Laptop present, Mouse appears 85.7% of the time
rule "Mined_Laptop_Implies_Mouse" {
    salience 85
    when
        ShoppingCart.contains("Laptop")
    then
        Recommendation.add("Mouse");
        LogMessage("Rule fired: Mined_Laptop_Implies_Mouse (confidence: 85.7%)");
}

// Rule #2: Laptop, Mouse → Laptop_Bag
// Confidence: 75.0% | Support: 45.0% | Lift: 1.88 | Conviction: 1.71
// Interpretation: When Laptop, Mouse present, Laptop_Bag appears 75.0% of the time
rule "Mined_Laptop_Mouse_Implies_Laptop_Bag" {
    salience 75
    when
        ShoppingCart.contains("Laptop") &&
        ShoppingCart.contains("Mouse")
    then
        Recommendation.add("Laptop_Bag");
        LogMessage("Rule fired: Mined_Laptop_Mouse_Implies_Laptop_Bag (confidence: 75.0%)");
}
```

**Success Criteria:**
- Generate syntactically correct GRL code
- Include metrics as comments for transparency
- Loadable into RuleEngine without errors

---

### Stage 6: Graph Pattern Matching (Week 8)

```rust
impl RuleMiner {
    /// Find graph patterns (subgraph isomorphism)
    pub fn find_graph_patterns(&self, template: &Graph<Entity, Relationship>) -> Vec<GraphPattern> {
        use petgraph::algo::isomorphism::subgraph_isomorphisms_iter;

        let mut patterns = Vec::new();

        // Find all subgraphs matching the template
        for mapping in subgraph_isomorphisms_iter(template, &self.entity_graph) {
            let matched_nodes: Vec<NodeIndex> = mapping.values().cloned().collect();

            let pattern = GraphPattern {
                nodes: matched_nodes.iter()
                    .map(|&idx| self.entity_graph[idx].clone())
                    .collect(),
                edges: self.extract_edges(&matched_nodes),
                support: 0.0, // Calculate based on occurrence
            };

            patterns.push(pattern);
        }

        patterns
    }

    /// Example: Find "influencer" pattern
    /// User with high follower count who makes purchases that others follow
    pub fn find_influencer_pattern(&self) -> Vec<GraphPattern> {
        // Template: User → follows → User → purchases → Product
        let mut template = Graph::new();

        let user1 = template.add_node(Entity {
            id: "user1".to_string(),
            entity_type: EntityType::User,
            attributes: HashMap::new(),
        });

        let user2 = template.add_node(Entity {
            id: "user2".to_string(),
            entity_type: EntityType::User,
            attributes: HashMap::new(),
        });

        let product = template.add_node(Entity {
            id: "product".to_string(),
            entity_type: EntityType::Product,
            attributes: HashMap::new(),
        });

        template.add_edge(user2, user1, Relationship {
            rel_type: RelationType::Custom("follows".to_string()),
            confidence: 1.0,
            support: 0,
            lift: 1.0,
            time_window: None,
            metadata: HashMap::new(),
        });

        template.add_edge(user1, product, Relationship {
            rel_type: RelationType::Custom("purchases".to_string()),
            confidence: 1.0,
            support: 0,
            lift: 1.0,
            time_window: None,
            metadata: HashMap::new(),
        });

        self.find_graph_patterns(&template)
    }
}
```

**Success Criteria:**
- Match complex graph patterns (3+ nodes)
- Support custom template patterns
- Use petgraph's isomorphism algorithms efficiently

---

### Stage 7: Integration & Testing (Week 9-10)

**Integration with RuleEngine:**
```rust
// Complete workflow example
use rust_rule_engine::mining::RuleMiner;
use rust_rule_engine::RuleEngine;

// 1. Mine rules from data
let mut miner = RuleMiner::new()
    .with_min_support(0.1)
    .with_min_confidence(0.7)
    .with_min_lift(1.2);

miner.load_from_json("historical_transactions.json")?;

let rules = miner.mine_association_rules()?;
let grl_code = miner.to_grl(&rules);

// 2. Save for review
std::fs::write("mined_rules.grl", &grl_code)?;

// 3. Load into engine
let mut engine = RuleEngine::new();
engine.add_rules_from_grl(&grl_code)?;

// 4. Use for recommendations
let mut facts = Facts::new();
facts.set("ShoppingCart.items", vec!["Laptop"]);
engine.execute(&mut facts)?;

// Results: Recommendation.items = ["Mouse", "Laptop_Bag"]
println!("Recommendations: {:?}", facts.get("Recommendation.items"));
```

**Validation with Backward Chaining:**
```rust
use rust_rule_engine::backward::BackwardEngine;

// Verify mined rule makes sense
let mut bc_engine = BackwardEngine::new(kb);

// Query: "Should we recommend Mouse given Laptop purchase?"
let result = bc_engine.query(
    "should_recommend(?product) WHERE
        cart_contains(Laptop) AND
        high_correlation(Laptop, ?product) AND
        NOT already_purchased(?product)",
    &mut facts
)?;

// Result should include Mouse with high confidence
assert!(result.bindings.iter().any(|b| b.get("product") == Some("Mouse")));
```

**Testing Strategy:**

1. **Unit Tests:**
```rust
#[test]
fn test_apriori_basic() {
    let miner = create_test_miner();
    let itemsets = miner.find_frequent_itemsets().unwrap();

    // Should find {Laptop}, {Mouse}, {Laptop, Mouse}
    assert!(itemsets.iter().any(|(s, _)| s == &vec!["Laptop"]));
    assert!(itemsets.iter().any(|(s, _)| s == &vec!["Mouse"]));
    assert!(itemsets.iter().any(|(s, _)| s == &vec!["Laptop", "Mouse"]));
}

#[test]
fn test_association_rules() {
    let miner = create_test_miner();
    let rules = miner.generate_association_rules().unwrap();

    // Should find Laptop → Mouse with high confidence
    let laptop_to_mouse = rules.iter()
        .find(|r| r.antecedent == vec!["Laptop"] && r.consequent == vec!["Mouse"]);

    assert!(laptop_to_mouse.is_some());
    assert!(laptop_to_mouse.unwrap().metrics.confidence > 0.7);
}

#[test]
fn test_grl_generation() {
    let miner = create_test_miner();
    let rules = miner.generate_association_rules().unwrap();
    let grl = miner.to_grl(&rules);

    // Should be valid GRL
    assert!(grl.contains("rule"));
    assert!(grl.contains("when"));
    assert!(grl.contains("then"));

    // Should be parseable
    let mut engine = RuleEngine::new();
    assert!(engine.add_rules_from_grl(&grl).is_ok());
}
```

2. **Integration Tests:**
```rust
#[test]
fn test_end_to_end_mining() {
    // Load real dataset
    let miner = RuleMiner::new();
    miner.load_from_json("tests/data/ecommerce_transactions.json").unwrap();

    // Mine patterns
    let patterns = miner.mine_association_rules().unwrap();
    assert!(patterns.len() > 0);

    // Generate GRL
    let grl = miner.to_grl(&patterns);

    // Load into engine
    let mut engine = RuleEngine::new();
    engine.add_rules_from_grl(&grl).unwrap();

    // Test recommendation
    let mut facts = Facts::new();
    facts.set("ShoppingCart.items", vec!["Laptop"]);
    engine.execute(&mut facts).unwrap();

    // Should have recommendations
    let recs = facts.get("Recommendation.items").unwrap();
    assert!(recs.as_array().unwrap().len() > 0);
}
```

3. **Benchmarks:**
```rust
#[bench]
fn bench_apriori_1k_transactions(b: &mut Bencher) {
    let miner = create_miner_with_transactions(1_000);
    b.iter(|| {
        miner.find_frequent_itemsets()
    });
}

#[bench]
fn bench_apriori_10k_transactions(b: &mut Bencher) {
    let miner = create_miner_with_transactions(10_000);
    b.iter(|| {
        miner.find_frequent_itemsets()
    });
}
```

**Success Criteria:**
- All unit tests pass
- End-to-end workflow works
- Performance acceptable (<10s for 10k transactions)

---

## 🎯 Performance Targets

| Scenario | Target Performance | Notes |
|----------|-------------------|-------|
| Load 10,000 transactions | <500ms | JSON parsing + indexing |
| Build entity graph | <1s | 10k nodes, 50k edges |
| Apriori (1k transactions, 100 items) | <2s | Classic algorithm |
| FP-Growth (10k transactions, 1000 items) | <5s | Optimized algorithm |
| Generate GRL (100 rules) | <100ms | String formatting |
| Sequential pattern mining | <10s | Time-window based |

---

## 🧪 Example Use Cases

### 1. E-commerce Product Recommendations
```rust
// Input: Purchase history
let transactions = load_amazon_data();

// Mine patterns
let miner = RuleMiner::new()
    .with_min_confidence(0.75)
    .with_min_support(0.05);  // 5% of customers

let rules = miner.mine_association_rules()?;

// Output: "Customers who bought X also bought Y" rules
// Example: Laptop (85%) → Mouse, Keyboard (75%) → Monitor
```

### 2. Fraud Detection Pattern Discovery
```rust
// Input: Fraud cases + normal transactions
let fraud_transactions = load_fraud_cases();
let normal_transactions = load_normal_transactions();

// Mine patterns unique to fraud cases
let fraud_miner = RuleMiner::new().with_transactions(fraud_transactions);
let normal_miner = RuleMiner::new().with_transactions(normal_transactions);

let fraud_patterns = fraud_miner.mine_association_rules()?;
let normal_patterns = normal_miner.mine_association_rules()?;

// Find patterns that appear in fraud but not normal
let unique_fraud_patterns = fraud_patterns.iter()
    .filter(|p| !normal_patterns.contains(p))
    .collect();

// Output: Rules like "IP mismatch + unusual time + high amount → fraud"
```

### 3. Medical Diagnosis Support
```rust
// Input: Patient symptoms + confirmed diagnoses
let patient_records = load_medical_records();

let miner = RuleMiner::new()
    .with_min_confidence(0.90)  // High confidence for medical
    .with_min_support(0.02);    // Even rare diseases

let rules = miner.mine_association_rules()?;

// Output: "Symptoms A, B, C → Likely Disease X (90% confidence)"
```

### 4. Network Security Attack Patterns
```rust
// Input: Network traffic logs + attack labels
let attack_logs = load_attack_logs();

let miner = RuleMiner::new();
miner.load_sequential_data(attack_logs)?;

let sequential_patterns = miner.find_sequential_patterns()?;

// Output: "Port scan → Login attempt → Data exfiltration" patterns
```

---

## 🚧 Challenges & Solutions

### Challenge 1: Computational Complexity
**Problem:** Apriori has exponential worst-case complexity
**Solution:**
- Use FP-Growth for large datasets (linear in dataset size)
- Implement early pruning heuristics
- Parallel processing for independent itemset generation
- Cache frequent itemsets

### Challenge 2: Too Many Low-Quality Rules
**Problem:** Combinatorial explosion of rules with low business value
**Solution:**
- Aggressive filtering: min_confidence=0.7, min_lift=1.2
- Rank by conviction (not just confidence)
- User-defined "interesting" templates
- Top-K selection instead of all rules

### Challenge 3: Temporal Pattern Complexity
**Problem:** Sequential patterns with time constraints are expensive
**Solution:**
- Limit max sequence length (default: 3 steps)
- Use time-window bucketing (hourly/daily instead of exact timestamps)
- Approximate patterns for exploration, exact for validation

### Challenge 4: Graph Pattern Matching Performance
**Problem:** Subgraph isomorphism is NP-complete
**Solution:**
- Use petgraph's optimized algorithms
- Limit template size (<5 nodes)
- Use heuristics (degree filtering, label matching)
- Approximate matching for large graphs

### Challenge 5: Domain-Specific Semantics
**Problem:** Generic patterns may not make business sense
**Solution:**
- Allow user-defined pattern templates
- Category hierarchies (e.g., Electronics → Laptop)
- Negative patterns (never recommend together)
- Domain-specific metrics (beyond confidence/support)

---

## 📊 Success Metrics

### Functional Metrics
- ✅ Discover 10+ high-quality rules from 1,000 transactions
- ✅ Support min_support range: 0.01 - 0.5
- ✅ Support min_confidence range: 0.5 - 1.0
- ✅ Generate valid GRL code (100% parse success)

### Performance Metrics
- ✅ <10 seconds for 10,000 transactions (Apriori)
- ✅ <5 seconds for 10,000 transactions (FP-Growth)
- ✅ Memory usage <500MB for 10k transactions

### Quality Metrics
- ✅ Mined rules improve recommendation accuracy >20%
- ✅ Lift scores >1.5 for top-10 rules
- ✅ Conviction scores >2.0 for actionable rules

### Usability Metrics
- ✅ <5 lines of code for basic workflow
- ✅ JSON/CSV input support (no custom formats)
- ✅ Human-readable GRL output with explanations

---

## 🔄 Integration with Existing Features

### With Forward Chaining (RETE)
```rust
// Mined rules execute in RETE for performance
let grl = miner.to_grl(&rules);
let mut engine = RuleEngine::new_with_rete();
engine.add_rules_from_grl(&grl)?;

// Benefit: O(1) rule matching + mined knowledge
```

### With Backward Chaining
```rust
// Use backward chaining to explain WHY a rule was mined
let bc_engine = BackwardEngine::new(kb);

// Query: "Why should we recommend Mouse for Laptop?"
let explanation = bc_engine.query(
    "explain_recommendation(Mouse, Laptop)",
    &mut facts
)?;

// Result: Proof tree showing co-occurrence statistics
```

### With Stream Processing
```rust
// Future: Online mining from event streams
let stream_miner = StreamRuleMiner::new();

stream_miner.process_event_stream("user_actions", |event| {
    // Update pattern counts incrementally
    stream_miner.update_patterns(event);

    // Trigger rule regeneration when patterns shift
    if stream_miner.pattern_drift_detected() {
        let new_rules = stream_miner.mine_current_patterns()?;
        engine.reload_rules(&new_rules)?;
    }
});
```

---

## 🔄 Future Enhancements (Phase 2)

### 1. Advanced Algorithms
- **FP-Growth**: Faster than Apriori for large datasets
- **Eclat**: Vertical data format, good for sparse data
- **SPADE**: Sequential pattern discovery (faster than Apriori-based)
- **GSP**: Generalized sequential patterns

### 2. Multi-Level Mining
```rust
// Mine at different abstraction levels
// Level 1: "Laptop" → "Mouse"
// Level 2: "Electronics" → "Accessories"
// Level 3: "High-value items" → "Complementary low-value items"

miner.set_taxonomy(taxonomy_tree);
let hierarchical_rules = miner.mine_hierarchical_patterns()?;
```

### 3. Negative Pattern Mining
```rust
// Find what DOESN'T happen together
// Example: "iPhone buyers rarely buy Android accessories"

let negative_rules = miner.mine_negative_associations()?;
// Output: iPhone → NOT(Android_Charger) [confidence: 95%]
```

### 4. Incremental Mining
```rust
// Update patterns as new data arrives (no full re-scan)
miner.add_transaction_incremental(new_transaction)?;

// Only recompute affected itemsets
let updated_rules = miner.get_incremental_updates()?;
```

### 5. Distributed Mining
```rust
// Parallelize across multiple machines
let distributed_miner = DistributedRuleMiner::new(cluster_config);
distributed_miner.partition_data(data_shards)?;

// Each node mines local patterns, then merge
let global_rules = distributed_miner.mine_distributed()?;
```

### 6. Visualization & Explainability
```rust
// Interactive graph visualization
miner.export_graph_html("patterns.html")?;

// Explanation for each rule
let explanation = miner.explain_rule(&rule)?;
// Returns: supporting transactions, counter-examples, alternative patterns
```

### 7. A/B Testing Integration
```rust
// Test mined rules vs manual rules
let ab_test = ABTestRunner::new();
ab_test.add_variant("mined_rules", mined_grl);
ab_test.add_variant("manual_rules", manual_grl);

let results = ab_test.run_for_duration(Duration::days(7))?;
// Returns: conversion rates, recommendation acceptance, revenue impact
```

---

## 🗓️ Timeline

**Total Duration:** 10 weeks

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1-2 | Data Input & Graph | Parse transactions, build graph |
| 3-4 | Frequent Itemsets | Apriori algorithm implementation |
| 5 | Association Rules | Confidence/support/lift calculation |
| 6 | Sequential Patterns | Time-based pattern mining |
| 7 | GRL Generation | Convert patterns → GRL code |
| 8 | Graph Patterns | Subgraph matching with petgraph |
| 9-10 | Integration & Testing | End-to-end workflow + benchmarks |

**Release:** v1.15.0 (Graph-Based Rule Mining)

---

## 📚 References & Further Reading

### Classic Papers
1. **"Fast Algorithms for Mining Association Rules"** - Agrawal & Srikant (VLDB 1994)
   - The original Apriori algorithm paper

2. **"Mining Frequent Patterns without Candidate Generation"** - Han et al. (SIGMOD 2000)
   - FP-Growth algorithm

3. **"Mining Sequential Patterns"** - Agrawal & Srikant (ICDE 1995)
   - Sequential pattern mining

### Books
- **"Data Mining: Concepts and Techniques"** - Han, Kamber, Pei
- **"Introduction to Data Mining"** - Tan, Steinbach, Kumar
- **"The Elements of Statistical Learning"** - Hastie, Tibshirani, Friedman

### Similar Tools
- **MLxtend** (Python): Association rule mining library
- **Weka** (Java): Data mining workbench
- **Orange** (Python): Visual data mining

---

## ✅ Next Steps

1. **Review & Approve** - Get stakeholder feedback on design
2. **Create POC** - Build basic Apriori implementation (Week 1-2 scope)
3. **Benchmark POC** - Validate performance on sample datasets
4. **Iterate Design** - Refine based on POC learnings
5. **Full Implementation** - Execute 10-week plan

---

**Last Updated:** 2026-01-02
**Author:** Ton That Vu
**Status:** Planning Phase - Ready for Review
**Complexity:** High
**Priority:** Medium-High
