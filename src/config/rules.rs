use std::collections::HashMap;
use std::fmt;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use toml::Value;

/// A rule that conditionally enforces constraints on field values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Human-readable name for the rule (used in error messages)
    pub name: Option<String>,

    /// Condition that must be true for the rule's actions to apply
    pub condition: Expr,

    /// Actions to enforce when the condition is met
    pub action: Vec<Action>,
}

/// An action to enforce when a rule's condition is met.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// These fields must be non-empty
    Require {
        fields: Vec<String>,
        message: Option<String>,
    },
    /// These fields must be empty
    Forbid {
        fields: Vec<String>,
        message: Option<String>,
    },
    /// These fields must equal the given value
    Equals {
        fields: Vec<String>,
        value: Value,
        message: Option<String>,
    },
}

/// A boolean expression tree for rule conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Expr {
    // Logical combinators
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),

    // Comparisons
    Eq(String, Value),
    Neq(String, Value),

    // Set membership
    In(String, Vec<Value>),

    // Field presence (non-empty)
    Exists(String),
}

type Context = HashMap<String, Value>;

impl Expr {
    /// Evaluate this expression against a context of field values.
    pub fn evaluate(&self, ctx: &Context) -> bool {
        match self {
            Expr::And(a, b) => a.evaluate(ctx) && b.evaluate(ctx),
            Expr::Or(a, b) => a.evaluate(ctx) || b.evaluate(ctx),
            Expr::Not(e) => !e.evaluate(ctx),

            Expr::Eq(field, value) => ctx.get(field) == Some(value),
            Expr::Neq(field, value) => ctx.get(field) != Some(value),

            Expr::In(field, values) => ctx.get(field).map_or(false, |v| values.contains(v)),

            Expr::Exists(field) => ctx.contains_key(field),
        }
    }

    /// Collect all field names referenced by this expression (for config validation).
    pub fn referenced_fields(&self) -> Vec<&str> {
        match self {
            Expr::And(a, b) | Expr::Or(a, b) => {
                let mut fields = a.referenced_fields();
                fields.extend(b.referenced_fields());
                fields
            }
            Expr::Not(e) => e.referenced_fields(),
            Expr::Eq(f, _) | Expr::Neq(f, _) | Expr::In(f, _) | Expr::Exists(f) => {
                vec![f.as_str()]
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Eq(field, value) => write!(f, "{}={}", field, value),
            Expr::Neq(field, value) => write!(f, "{}!={}", field, value),
            Expr::In(field, values) => write!(f, "{} in {:?}", field, values),
            Expr::Exists(field) => write!(f, "{} exists", field),
            Expr::And(a, b) => write!(f, "{} and {}", a, b),
            Expr::Or(a, b) => write!(f, "{} or {}", a, b),
            Expr::Not(e) => write!(f, "not {}", e),
        }
    }
}

impl Action {
    /// Get the field names referenced by this action.
    pub fn referenced_fields(&self) -> Vec<&str> {
        match self {
            Action::Require { fields, .. }
            | Action::Forbid { fields, .. }
            | Action::Equals { fields, .. } => fields.iter().map(|f| f.as_str()).collect(),
        }
    }
    fn check(
        &self,
        rule_name: &Option<String>,
        condition: &Expr,
        values: &HashMap<String, String>,
        violations: &mut Vec<RuleViolation>,
    ) {
        match self {
            Action::Require { fields, message } => {
                for field in fields {
                    if values.get(field).map_or(true, |v| v.is_empty()) {
                        violations.push(RuleViolation {
                            rule_name: rule_name.clone(),
                            message: message.clone().unwrap_or_else(|| {
                                format!("When {}: field '{}' is required", condition, field)
                            }),
                        });
                    }
                }
            }
            Action::Forbid { fields, message } => {
                for field in fields {
                    if values.get(field).map_or(false, |v| !v.is_empty()) {
                        violations.push(RuleViolation {
                            rule_name: rule_name.clone(),
                            message: message.clone().unwrap_or_else(|| {
                                format!("When {}: field '{}' is forbidden", condition, field)
                            }),
                        });
                    }
                }
            }
            Action::Equals {
                fields,
                value,
                message,
            } => {
                for field in fields {
                    let field_value = values.get(field).map(|v| Value::String(v.clone()));
                    if field_value.as_ref() != Some(value) {
                        violations.push(RuleViolation {
                            rule_name: rule_name.clone(),
                            message: message.clone().unwrap_or_else(|| {
                                format!(
                                    "When {}: field '{}' must equal {}",
                                    condition, field, value
                                )
                            }),
                        });
                    }
                }
            }
        }
    }
}

/// A single rule violation with context for error reporting.
#[derive(Debug)]
pub(crate) struct RuleViolation {
    pub(crate) rule_name: Option<String>,
    pub(crate) message: String,
}

impl fmt::Display for RuleViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.rule_name {
            Some(name) => write!(f, "[{}] {}", name, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

/// Build a rule evaluation context from collected field values.
///
/// Empty values are excluded so that `Exists` checks work correctly —
/// a field only "exists" if the user actually provided a value.
fn build_context(values: &HashMap<String, String>) -> Context {
    values
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect()
}

/// Evaluate all rules against the collected field values.
///
/// Returns `Ok(())` if all rules pass, or an error listing all violations.
pub fn evaluate_rules(rules: &[Rule], values: &HashMap<String, String>) -> Result<()> {
    let ctx = build_context(values);
    let mut violations = Vec::new();

    for rule in rules {
        if rule.condition.evaluate(&ctx) {
            for action in &rule.action {
                action.check(&rule.name, &rule.condition, values, &mut violations);
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        let messages: Vec<String> = violations.iter().map(|v| v.to_string()).collect();
        bail!("Rule violations:\n  • {}", messages.join("\n  • "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(pairs: &[(&str, &str)]) -> Context {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), Value::String(v.to_string())))
            .collect()
    }

    fn values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn rule(name: &str, condition: Expr, action: Vec<Action>) -> Rule {
        Rule {
            name: Some(name.into()),
            condition,
            action,
        }
    }

    // ── Expr evaluation ──────────────────────────────────────

    #[test]
    fn test_expr_eq() {
        let expr = Expr::Eq("type".into(), Value::String("feat".into()));
        assert!(expr.evaluate(&ctx(&[("type", "feat")])));
        assert!(!expr.evaluate(&ctx(&[("type", "fix")])));
    }

    #[test]
    fn test_expr_neq() {
        let expr = Expr::Neq("type".into(), Value::String("feat".into()));
        assert!(!expr.evaluate(&ctx(&[("type", "feat")])));
        assert!(expr.evaluate(&ctx(&[("type", "fix")])));
    }

    #[test]
    fn test_expr_in() {
        let expr = Expr::In(
            "type".into(),
            vec![Value::String("feat".into()), Value::String("fix".into())],
        );
        assert!(expr.evaluate(&ctx(&[("type", "feat")])));
        assert!(expr.evaluate(&ctx(&[("type", "fix")])));
        assert!(!expr.evaluate(&ctx(&[("type", "docs")])));
    }

    #[test]
    fn test_expr_exists() {
        let expr = Expr::Exists("scope".into());
        assert!(expr.evaluate(&ctx(&[("scope", "api")])));
        assert!(!expr.evaluate(&ctx(&[])));
    }

    #[test]
    fn test_expr_and() {
        let expr = Expr::And(
            Box::new(Expr::Eq("type".into(), Value::String("feat".into()))),
            Box::new(Expr::Exists("scope".into())),
        );
        assert!(expr.evaluate(&ctx(&[("type", "feat"), ("scope", "api")])));
        assert!(!expr.evaluate(&ctx(&[("type", "feat")])));
    }

    #[test]
    fn test_expr_or() {
        let expr = Expr::Or(
            Box::new(Expr::Eq("type".into(), Value::String("feat".into()))),
            Box::new(Expr::Eq("type".into(), Value::String("fix".into()))),
        );
        assert!(expr.evaluate(&ctx(&[("type", "feat")])));
        assert!(expr.evaluate(&ctx(&[("type", "fix")])));
        assert!(!expr.evaluate(&ctx(&[("type", "docs")])));
    }

    #[test]
    fn test_expr_not() {
        let expr = Expr::Not(Box::new(Expr::Exists("scope".into())));
        assert!(expr.evaluate(&ctx(&[])));
        assert!(!expr.evaluate(&ctx(&[("scope", "api")])));
    }

    // ── build_context ────────────────────────────────────────

    #[test]
    fn test_build_context_excludes_empty_values() {
        let vals = values(&[("type", "feat"), ("scope", ""), ("desc", "hello")]);
        let ctx = build_context(&vals);
        assert!(ctx.contains_key("type"));
        assert!(!ctx.contains_key("scope"));
        assert!(ctx.contains_key("desc"));
    }

    // ── Rule evaluation: Require ─────────────────────────────

    #[test]
    fn test_require_passes_when_field_present() {
        let rules = vec![rule(
            "require-scope-for-feat",
            Expr::Eq("type".into(), Value::String("feat".into())),
            vec![Action::Require {
                fields: vec!["scope".into()],
                message: None,
            }],
        )];
        assert!(evaluate_rules(&rules, &values(&[("type", "feat"), ("scope", "api")])).is_ok());
    }

    #[test]
    fn test_require_fails_when_field_empty() {
        let rules = vec![rule(
            "require-scope-for-feat",
            Expr::Eq("type".into(), Value::String("feat".into())),
            vec![Action::Require {
                fields: vec!["scope".into()],
                message: None,
            }],
        )];
        assert!(evaluate_rules(&rules, &values(&[("type", "feat"), ("scope", "")])).is_err());
    }

    // ── Rule evaluation: Forbid ──────────────────────────────

    #[test]
    fn test_forbid_passes_when_field_empty() {
        let rules = vec![rule(
            "no-scope-for-chore".into(),
            Expr::Eq("type".into(), Value::String("chore".into())),
            vec![Action::Forbid {
                fields: vec!["scope".into()],
                message: None,
            }],
        )];
        assert!(evaluate_rules(&rules, &values(&[("type", "chore"), ("scope", "")])).is_ok());
    }

    #[test]
    fn test_forbid_fails_when_field_present() {
        let rules = vec![rule(
            "no-scope-for-chore",
            Expr::Eq("type".into(), Value::String("chore".into())),
            vec![Action::Forbid {
                fields: vec!["scope".into()],
                message: None,
            }],
        )];
        assert!(
            evaluate_rules(
                &rules,
                &values(&[("type", "chore"), ("scope", "something")])
            )
            .is_err()
        );
    }

    // ── Rule evaluation: Equals ──────────────────────────────

    #[test]
    fn test_equals_passes() {
        let rules = vec![rule(
            "breaking-must-be-feat",
            Expr::Exists("breaking".into()),
            vec![Action::Equals {
                fields: vec!["type".into()],
                value: Value::String("feat".into()),
                message: None,
            }],
        )];
        assert!(evaluate_rules(&rules, &values(&[("type", "feat"), ("breaking", "yes")])).is_ok());
    }

    #[test]
    fn test_equals_fails() {
        let rules = vec![rule(
            "breaking-must-be-feat",
            Expr::Exists("breaking".into()),
            vec![Action::Equals {
                fields: vec!["type".into()],
                value: Value::String("feat".into()),
                message: None,
            }],
        )];
        assert!(evaluate_rules(&rules, &values(&[("type", "fix"), ("breaking", "yes")])).is_err());
    }

    // ── Condition not met → rule skipped ─────────────────────

    #[test]
    fn test_rule_skipped_when_condition_false() {
        let rules = vec![rule(
            "require-scope-for-feat",
            Expr::Eq("type".into(), Value::String("feat".into())),
            vec![Action::Require {
                fields: vec!["scope".into()],
                message: None,
            }],
        )];
        // type=fix, not feat → rule doesn't fire even though scope is empty
        assert!(evaluate_rules(&rules, &values(&[("type", "fix"), ("scope", "")])).is_ok());
    }

    // ── Multiple violations ──────────────────────────────────

    #[test]
    fn test_multiple_violations_reported() {
        let rules = vec![rule(
            "strict-feat",
            Expr::Eq("type".into(), Value::String("feat".into())),
            vec![
                Action::Require {
                    fields: vec!["scope".into()],
                    message: None,
                },
                Action::Require {
                    fields: vec!["body".into()],
                    message: None,
                },
            ],
        )];
        let err = evaluate_rules(
            &rules,
            &values(&[("type", "feat"), ("scope", ""), ("body", "")]),
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("scope"));
        assert!(msg.contains("body"));
    }
}
