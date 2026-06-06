use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Output template for formatting the final commit message
    pub output: OutputConfig,

    /// List of fields to prompt for
    #[serde(rename = "field", default)]
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

    /// For confirm fields: map true/false to custom strings
    #[serde(default)]
    pub values: Option<ConfirmValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmValues {
    #[serde(rename = "true")]
    pub on_true: String,
    #[serde(rename = "false")]
    pub on_false: String,
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
        self.validate_template_fields()?;
        self.validate_select_options()?;
        Ok(())
    }

    fn validate_template_fields(&self) -> Result<()> {
        // Extract placeholders from template
        let placeholder_regex = regex::Regex::new(r"\{([^}]+)\}").unwrap();
        let placeholders: HashSet<String> = placeholder_regex
            .captures_iter(&self.output.template)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        let known_fields: HashSet<String> = self.fields.iter().map(|f| f.id.clone()).collect();

        let undefined: Vec<String> = placeholders.difference(&known_fields).cloned().collect();
        if !undefined.is_empty() {
            bail!(
                "Template contains undefined placeholders: {}",
                undefined.join(", ")
            )
        }

        let unused: Vec<String> = known_fields.difference(&placeholders).cloned().collect();
        if !unused.is_empty() {
            bail!("Config defines unused fields: {}", unused.join(", "))
        }

        Ok(())
    }

    fn validate_select_options(&self) -> Result<()> {
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
}
