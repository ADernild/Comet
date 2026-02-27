use std::collections::HashSet;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Output template for formatting the final commit message
    pub output: OutputConfig,

    /// List of fields to prompt for
    #[serde(rename = "field")]
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Template string with placeholders like {type}, {scope}, {description}
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// Internal identifier for the field (used in template)
    pub id: String,

    /// Type of field (select, text, multiline, confirm)
    #[serde(rename = "type")]
    pub field_type: FieldType,

    /// Prompt text shown to user
    pub prompt: String,

    /// Whether this field is required
    #[serde(default)]
    pub required: bool,

    /// Help message shown to user
    #[serde(default)]
    pub help: Option<String>,

    /// Options for select fields
    #[serde(default)]
    pub options: Option<Vec<String>>,

    /// Validation rules
    pub validate: Option<Validation>,

    /// Whether to wrap text at a specific width (for multiline fields)
    #[serde(default)]
    pub wrap: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Select,
    Text,
    Multiline,
    Confirm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    /// Minimum length
    #[serde(default)]
    pub min: Option<usize>,

    /// Maximum length
    #[serde(default)]
    pub max: Option<usize>,

    /// Regex pattern
    #[serde(default)]
    pub pattern: Option<String>,
}

impl Config {
    /// Validate that all template placeholders have corresponding fields
    pub fn validate(&self) -> Result<()> {
        // Extract placeholders from template
        let placeholder_regex = regex::Regex::new(r"\{([^}]+)\}").unwrap();
        let placeholders: HashSet<String> = placeholder_regex
            .captures_iter(&self.output.template)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        let known_fields: HashSet<String> = self.fields.iter().map(|f| f.id.clone()).collect();

        // Check for undefined placeholders in template
        let undefined: Vec<String> = placeholders.difference(&known_fields).cloned().collect();
        if !undefined.is_empty() {
            bail!(
                "Template contains undefined placeholders: {}",
                undefined.join(", ")
            )
        }

        // Check for unused fields
        let unused: Vec<String> = known_fields.difference(&placeholders).cloned().collect();
        if !unused.is_empty() {
            bail!("Config defines unused fields: {}", unused.join(", "))
        }

        // Validate that Select fields have options
        for field in &self.fields {
            if field.field_type == FieldType::Select {
                match &field.options {
                    None => bail!("Select field '{}' must have options", field.id),
                    Some(opts) if opts.is_empty() => {
                        bail!("Select field '{}' must have at least one option", field.id)
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Render the commit message using the template and field values
    pub fn render(&self, values: &std::collections::HashMap<String, String>) -> Result<String> {
        let mut output = self.output.template.clone();

        let optional_fields: HashSet<String> = self
            .fields
            .iter()
            .filter(|f| !f.required)
            .map(|f| f.id.clone())
            .collect();

        for (key, value) in values {
            let placeholder = format!("{{{}}}", key);

            if value.is_empty() {
                // If it's optional, remove the placeholder
                if optional_fields.contains(key) {
                    output = output.replace(&placeholder, "");
                } else {
                    // If it's required and empty, that's an error
                    bail!("Required field '{}' cannot be empty", key);
                }
            } else {
                output = output.replace(&placeholder, value);
            }
        }
        Ok(clean_output(&output))
    }
}

fn clean_output(text: &str) -> String {
    let mut result = text.to_string();

    result = result.replace("()", "");

    result = result
        .lines()
        .filter(|line| !line.trim().is_empty() || line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    while result.contains("\n\n\n") {
        result = result.replace("\n\n\n", "\n\n")
    }
    result = result.trim().to_string();

    result
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    // Test fixtures
    fn create_field(id: &str, required: bool, field_type: FieldType) -> Field {
        Field {
            id: id.to_string(),
            field_type,
            prompt: format!("{} field", id),
            required,
            help: None,
            options: None,
            validate: None,
            wrap: None,
        }
    }

    fn create_config(template: &str, field_ids: Vec<(&str, bool, FieldType)>) -> Config {
        Config {
            output: OutputConfig {
                template: template.to_string(),
            },
            fields: field_ids
                .into_iter()
                .map(|(id, required, field_type)| create_field(id, required, field_type))
                .collect(),
        }
    }

    fn create_values(pairs: Vec<(&str, &str)>) -> HashMap<String, String> {
        pairs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // Validation tests
    #[test]
    fn test_validate_rejects_undefined_placeholders() {
        let config = create_config(
            "{type}: {undefined_field}",
            vec![("type", true, FieldType::Text)],
        );

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_rejects_unused_fields() {
        let config = create_config(
            "{type}: {description}",
            vec![
                ("type", true, FieldType::Text),
                ("description", true, FieldType::Text),
                ("unused_field", false, FieldType::Text),
            ],
        );

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_accepts_valid_configs() {
        let config = create_config(
            "{type}({scope}): {description}",
            vec![
                ("type", true, FieldType::Text),
                ("scope", false, FieldType::Text),
                ("description", true, FieldType::Text),
            ],
        );

        assert!(config.validate().is_ok());
    }

    // Render tests
    #[test]
    fn test_render_with_all_fields() {
        let config = create_config(
            "{type}({scope}): {description}",
            vec![
                ("type", true, FieldType::Text),
                ("scope", false, FieldType::Text),
                ("description", true, FieldType::Text),
            ],
        );

        let values = create_values(vec![
            ("type", "feat"),
            ("scope", "api"),
            ("description", "add endpoint"),
        ]);

        let result = config.render(&values).unwrap();
        assert_eq!(result, "feat(api): add endpoint");
    }

    #[test]
    fn test_render_removes_empty_optional_field() {
        let config = create_config(
            "{type}({scope}): {description}",
            vec![
                ("type", true, FieldType::Text),
                ("scope", false, FieldType::Text),
                ("description", true, FieldType::Text),
            ],
        );
        let values = create_values(vec![
            ("type", "feat"),
            ("scope", ""),
            ("description", "add endpoint"),
        ]);

        let result = config.render(&values).unwrap();
        assert_eq!(result, "feat: add endpoint");
    }

    #[test]
    fn test_render_with_multiline_fields() {
        let config = create_config(
            "{type}: {description}\n\n{body}\n\n{footer}",
            vec![
                ("type", true, FieldType::Text),
                ("description", true, FieldType::Text),
                ("body", false, FieldType::Multiline),
                ("footer", false, FieldType::Text),
            ],
        );

        let values = create_values(vec![
            ("type", "feat"),
            ("description", "add feature"),
            ("body", "Detailed explanation"),
            ("footer", "Closes #123"),
        ]);

        let result = config.render(&values).unwrap();
        assert_eq!(
            result,
            "feat: add feature\n\nDetailed explanation\n\nCloses #123"
        );
    }

    #[test]
    fn test_render_removes_empty_optional_multiline_fields() {
        let config = create_config(
            "{type}: {description}\n\n{body}\n\n{footer}",
            vec![
                ("type", true, FieldType::Text),
                ("description", true, FieldType::Text),
                ("body", false, FieldType::Multiline),
                ("footer", false, FieldType::Text),
            ],
        );

        let values = create_values(vec![
            ("type", "feat"),
            ("description", "add feature"),
            ("body", ""),
            ("footer", ""),
        ]);

        let result = config.render(&values).unwrap();
        assert_eq!(result, "feat: add feature");
    }

    #[test]
    fn test_render_rejects_empty_required_fields() {
        let config = create_config(
            "{type}({scope}): {description}",
            vec![
                ("type", true, FieldType::Select),
                ("scope", false, FieldType::Multiline),
                ("description", true, FieldType::Text),
            ],
        );
        let values = create_values(vec![
            ("type", ""),
            ("scope", ""),
            ("description", "add feature"),
        ]);

        assert!(config.render(&values).is_err());
    }
}
