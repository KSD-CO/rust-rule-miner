use crate::types::AssociationRule;
use chrono::Utc;

/// Export association rules to GRL (Grule Rule Language) format
pub struct GrlExporter;

impl GrlExporter {
    /// Convert association rules to GRL code
    pub fn to_grl(rules: &[AssociationRule]) -> String {
        let mut grl = String::new();

        // Header
        grl.push_str("// Auto-generated rules from pattern mining\n");
        grl.push_str(&format!("// Generated: {}\n", Utc::now()));
        grl.push_str(&format!("// Total rules: {}\n", rules.len()));
        grl.push('\n');

        // Generate each rule
        for (idx, rule) in rules.iter().enumerate() {
            grl.push_str(&Self::rule_to_grl(rule, idx));
            grl.push('\n');
        }

        grl
    }

    /// Convert a single rule to GRL format
    fn rule_to_grl(rule: &AssociationRule, idx: usize) -> String {
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
            Self::generate_conditions_with_negation(&rule.antecedent, &rule.consequent),
            Self::generate_actions(&rule.consequent),
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
    fn generate_conditions(items: &[String]) -> String {
        let conditions: Vec<String> = items
            .iter()
            .map(|item| format!("ShoppingCart.items contains \"{}\"", item))
            .collect();

        conditions.join(" &&\n        ")
    }

    /// Generate actions from consequent items
    fn generate_actions(items: &[String]) -> String {
        items
            .iter()
            .map(|item| format!("Recommendation.items += \"{}\"", item))
            .collect::<Vec<_>>()
            .join(";\n        ")
    }

    /// Generate conditions that check both antecedent AND that consequent items are NOT in recommendations
    fn generate_conditions_with_negation(antecedent: &[String], consequent: &[String]) -> String {
        let mut conditions = Vec::new();

        // Check ShoppingCart contains antecedent items
        for item in antecedent {
            conditions.push(format!("ShoppingCart.items contains \"{}\"", item));
        }

        // Check Recommendation does NOT contain consequent items (prevents duplicates)
        for item in consequent {
            conditions.push(format!("!(Recommendation.items contains \"{}\")", item));
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
