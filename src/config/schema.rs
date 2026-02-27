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
    pub options: Vec<String>,

    /// Validation rules
    pub validate: Option<Validation>,

    /// Whether to wrap text at a specific width (for multiline fields)
    #[serde(default)]
    pub wrap: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

        Ok(())
    }

    /// Render the commit message using the template and field values
    pub fn render(&self, values: &std::collections::HashMap<String, String>) -> String {
        let mut output = self.output.template.clone();

        let optional_fields: HashSet<String> = self
            .fields
            .iter()
            .filter(|f| !f.required)
            .map(|f| f.id.clone())
            .collect();

        for (key, value) in values {
            let placeholder = format!("{{{}}}", key);

            if value.is_empty() && optional_fields.contains(key) {
                output = output.replace(&placeholder, "");
            } else {
                output = output.replace(&placeholder, value);
            }
        }
        clean_output(&output)
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
