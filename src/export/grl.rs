use crate::types::AssociationRule;
use chrono::Utc;

/// Rule template types for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleTemplate {
    /// Simple recommendation: IF cart has A THEN recommend B
    Recommendation,
    /// Alert/Warning: IF pattern detected THEN alert
    Alert,
    /// Classification: IF A THEN category = X
    Classification,
    /// Scoring: IF A THEN increase score
    Scoring,
    /// Validation: IF A AND NOT B THEN error
    Validation,
    /// Complex multi-action rules
    MultiAction,
    /// Fraud detection pattern
    FraudDetection,
    /// Inventory management
    InventoryAlert,
}

/// Configuration for GRL export
#[derive(Debug, Clone)]
pub struct GrlConfig {
    /// Field name for input items (e.g., "ShoppingCart.items", "Transaction.items")
    pub input_field: String,
    /// Field name for output/recommendation items (e.g., "Recommendation.items", "Suggestions.items")
    pub output_field: String,
    /// Rule template to use
    pub template: RuleTemplate,
    /// Custom action prefix (for MultiAction template)
    pub action_prefix: Option<String>,
}

impl Default for GrlConfig {
    fn default() -> Self {
        Self {
            input_field: "ShoppingCart.items".to_string(),
            output_field: "Recommendation.items".to_string(),
            template: RuleTemplate::Recommendation,
            action_prefix: None,
        }
    }
}

impl GrlConfig {
    /// Create a new GRL configuration
    pub fn new(input_field: impl Into<String>, output_field: impl Into<String>) -> Self {
        Self {
            input_field: input_field.into(),
            output_field: output_field.into(),
            template: RuleTemplate::Recommendation,
            action_prefix: None,
        }
    }

    /// Set the rule template
    pub fn with_template(mut self, template: RuleTemplate) -> Self {
        self.template = template;
        self
    }

    /// Set custom action prefix
    pub fn with_action_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.action_prefix = Some(prefix.into());
        self
    }

    /// Create config for shopping cart recommendations
    pub fn shopping_cart() -> Self {
        Self::default()
    }

    /// Create config for alert rules
    pub fn alert(input_field: impl Into<String>) -> Self {
        Self {
            input_field: input_field.into(),
            output_field: "AlertSystem".to_string(),
            template: RuleTemplate::Alert,
            action_prefix: None,
        }
    }

    /// Create config for classification rules
    pub fn classification(
        input_field: impl Into<String>,
        category_field: impl Into<String>,
    ) -> Self {
        Self {
            input_field: input_field.into(),
            output_field: category_field.into(),
            template: RuleTemplate::Classification,
            action_prefix: None,
        }
    }

    /// Create config for scoring rules
    pub fn scoring(input_field: impl Into<String>, score_field: impl Into<String>) -> Self {
        Self {
            input_field: input_field.into(),
            output_field: score_field.into(),
            template: RuleTemplate::Scoring,
            action_prefix: None,
        }
    }

    /// Create config for fraud detection
    pub fn fraud_detection(input_field: impl Into<String>) -> Self {
        Self {
            input_field: input_field.into(),
            output_field: "FraudScore".to_string(),
            template: RuleTemplate::FraudDetection,
            action_prefix: None,
        }
    }

    /// Create config for inventory alerts
    pub fn inventory_alert(input_field: impl Into<String>) -> Self {
        Self {
            input_field: input_field.into(),
            output_field: "InventoryAlert".to_string(),
            template: RuleTemplate::InventoryAlert,
            action_prefix: None,
        }
    }

    /// Create config for transaction analysis
    pub fn transaction() -> Self {
        Self {
            input_field: "Transaction.items".to_string(),
            output_field: "Analysis.recommendations".to_string(),
            template: RuleTemplate::Recommendation,
            action_prefix: None,
        }
    }

    /// Create config for custom fields
    pub fn custom(input_field: impl Into<String>, output_field: impl Into<String>) -> Self {
        Self::new(input_field, output_field)
    }
}

/// Export association rules to GRL (Grule Rule Language) format
pub struct GrlExporter;

impl GrlExporter {
    /// Convert association rules to GRL code (uses default config)
    pub fn to_grl(rules: &[AssociationRule]) -> String {
        Self::to_grl_with_config(rules, &GrlConfig::default())
    }

    /// Convert association rules to GRL code with custom configuration
    pub fn to_grl_with_config(rules: &[AssociationRule], config: &GrlConfig) -> String {
        let mut grl = String::new();

        // Header
        grl.push_str("// Auto-generated rules from pattern mining\n");
        grl.push_str(&format!("// Generated: {}\n", Utc::now()));
        grl.push_str(&format!("// Total rules: {}\n", rules.len()));
        grl.push_str(&format!("// Input field: {}\n", config.input_field));
        grl.push_str(&format!("// Output field: {}\n", config.output_field));
        grl.push('\n');

        // Generate each rule
        for (idx, rule) in rules.iter().enumerate() {
            grl.push_str(&Self::rule_to_grl(rule, idx, config));
            grl.push('\n');
        }

        grl
    }

    /// Convert a single rule to GRL format
    fn rule_to_grl(rule: &AssociationRule, idx: usize, config: &GrlConfig) -> String {
        let rule_name = Self::generate_rule_name(rule, idx);
        let salience = (rule.metrics.confidence * 100.0) as i32;

        format!(
            r#"// Rule #{}: {} => {}
// Confidence: {:.1}% | Support: {:.1}% | Lift: {:.2} | Conviction: {:.2}
// Interpretation: When {} present, {} appears {:.1}% of the time
rule "{}" salience {} no-loop {{
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
            Self::generate_conditions_with_negation(&rule.antecedent, &rule.consequent, config),
            Self::generate_actions(&rule.consequent, config),
            rule_name,
            rule.metrics.confidence * 100.0
        )
    }

    /// Generate rule name from antecedent and consequent
    fn generate_rule_name(rule: &AssociationRule, idx: usize) -> String {
        let antecedent_str = rule
            .antecedent
            .iter()
            .map(|s| s.replace(' ', "_"))
            .collect::<Vec<_>>()
            .join("_");

        let consequent_str = rule
            .consequent
            .iter()
            .map(|s| s.replace(' ', "_"))
            .collect::<Vec<_>>()
            .join("_");

        format!(
            "Mined_{}_{}_Implies_{}",
            idx, antecedent_str, consequent_str
        )
    }

    /// Generate conditions from antecedent and consequent items
    #[allow(dead_code)]
    fn generate_conditions(items: &[String], config: &GrlConfig) -> String {
        let conditions: Vec<String> = items
            .iter()
            .map(|item| format!("{} contains \"{}\"", config.input_field, item))
            .collect();

        conditions.join(" &&\n        ")
    }

    /// Generate actions from consequent items based on template
    fn generate_actions(items: &[String], config: &GrlConfig) -> String {
        match config.template {
            RuleTemplate::Recommendation => {
                // Add items to recommendation list
                items
                    .iter()
                    .map(|item| format!("{} += \"{}\"", config.output_field, item))
                    .collect::<Vec<_>>()
                    .join(";\n        ")
            }
            RuleTemplate::Alert => {
                // Generate alert with items list
                let items_str = items.join(", ");
                format!(
                    "{}.triggerAlert(\"Pattern detected: {}\");\n        {}.addDetails(\"Items: {}\")",
                    config.output_field, items_str, config.output_field, items_str
                )
            }
            RuleTemplate::Classification => {
                // Set classification category
                let default_category = "Unknown".to_string();
                let category = items.first().unwrap_or(&default_category);
                format!(
                    "{}.category = \"{}\";\n        {}.confidence = 1.0",
                    config.output_field, category, config.output_field
                )
            }
            RuleTemplate::Scoring => {
                // Increase score for each item
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let weight = 10.0 / (i + 1) as f64; // Decreasing weights
                        format!("{} += {:.1}; // {}", config.output_field, weight, item)
                    })
                    .collect::<Vec<_>>()
                    .join(";\n        ")
            }
            RuleTemplate::Validation => {
                // Generate validation error
                let items_str = items.join(", ");
                format!(
                    "ValidationError.addError(\"Missing required items: {}\");\n        ValidationError.setInvalid(true)",
                    items_str
                )
            }
            RuleTemplate::MultiAction => {
                // Multiple actions with custom prefix
                let prefix = config.action_prefix.as_deref().unwrap_or("Action");
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| format!("{}.step{}(\"{}\");", prefix, i + 1, item))
                    .collect::<Vec<_>>()
                    .join("\n        ")
            }
            RuleTemplate::FraudDetection => {
                // Fraud scoring and flagging
                let risk_score = items.len() * 15; // Higher for more indicators
                format!(
                    "{} += {};\n        FraudFlags.add({:?});\n        if ({} > 70) {{ Transaction.block(); }}",
                    config.output_field,
                    risk_score,
                    items,
                    config.output_field
                )
            }
            RuleTemplate::InventoryAlert => {
                // Inventory management alerts
                items
                    .iter()
                    .map(|item| {
                        format!(
                            "{}.createAlert(\"Low stock pattern: {}\");\n        PurchaseOrder.suggest(\"{}\")",
                            config.output_field, item, item
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(";\n        ")
            }
        }
    }

    /// Generate conditions that check both antecedent AND that consequent items are NOT in recommendations
    fn generate_conditions_with_negation(
        antecedent: &[String],
        consequent: &[String],
        config: &GrlConfig,
    ) -> String {
        let mut conditions = Vec::new();

        // Check input field contains antecedent items
        for item in antecedent {
            conditions.push(format!("{} contains \"{}\"", config.input_field, item));
        }

        // Check output field does NOT contain consequent items (prevents duplicates)
        for item in consequent {
            conditions.push(format!("!({} contains \"{}\")", config.output_field, item));
        }

        conditions.join(" &&\n        ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PatternMetrics;

    #[test]
    fn test_grl_generation() {
        let rule = AssociationRule {
            antecedent: vec!["Laptop".to_string()],
            consequent: vec!["Mouse".to_string()],
            metrics: PatternMetrics {
                confidence: 0.857,
                support: 0.6,
                lift: 1.43,
                conviction: 2.33,
                avg_time_gap: None,
                time_variance: None,
            },
        };

        let grl = GrlExporter::to_grl(&[rule]);

        assert!(grl.contains("rule"));
        assert!(grl.contains("Laptop"));
        assert!(grl.contains("Mouse"));
        assert!(grl.contains("85.7%"));
        assert!(grl.contains("ShoppingCart.items contains"));
        assert!(grl.contains("Recommendation.items +="));
    }

    #[test]
    fn test_multi_item_rule() {
        let rule = AssociationRule {
            antecedent: vec!["Laptop".to_string(), "Mouse".to_string()],
            consequent: vec!["USB Hub".to_string()],
            metrics: PatternMetrics {
                confidence: 0.75,
                support: 0.45,
                lift: 1.88,
                conviction: 1.71,
                avg_time_gap: None,
                time_variance: None,
            },
        };

        let grl = GrlExporter::to_grl(&[rule]);

        assert!(grl.contains("Laptop"));
        assert!(grl.contains("Mouse"));
        assert!(grl.contains("USB Hub"));
        assert!(grl.contains("&&")); // Multiple conditions
    }
}
