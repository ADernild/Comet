use crate::config::{Config, Field, FieldType};
use anyhow::Result;
use inquire::{InquireError, Select, Text, required, validator::Validation};
use std::collections::HashMap;

/// Prompt for commit message based on config
pub fn prompt_commit_from_config(config: &Config) -> Result<String> {
    let mut values: HashMap<String, String> = HashMap::new();

    // Prompt for each field in order
    for field in &config.fields {
        let mut value = prompt_field(field)?;

        // Wrap multiline fields if wrap is specified
        if matches!(field.field_type, FieldType::Multiline)
            && !value.is_empty()
            && let Some(width) = field.wrap
        {
            value = textwrap::fill(&value, width);
        }

        values.insert(field.id.clone(), value);
    }

    let commit_message = config.render(&values)?;
    Ok(commit_message)
}

/// Prompt for a single field based on its configuration
fn prompt_field(field: &Field) -> Result<String, InquireError> {
    match field.field_type {
        FieldType::Select => prompt_select(field),
        FieldType::Text | FieldType::Multiline => prompt_text(field),
        FieldType::Confirm => prompt_confirm(field),
    }
}

/// Prompt for select field
fn prompt_select(field: &Field) -> Result<String, InquireError> {
    let mut select = Select::new(&field.prompt, field.options.clone());

    if let Some(help) = &field.help {
        select = select.with_help_message(help);
    }
    let answer = select.prompt()?;
    Ok(answer)
}

/// Prompt for text field
fn prompt_text(field: &Field) -> Result<String, InquireError> {
    let mut text = Text::new(&field.prompt);

    if let Some(help) = &field.help {
        text = text.with_help_message(help);
    }

    if field.required {
        text = text.with_validator(required!())
    }

    if let Some(validation) = &field.validate {
        if let Some(min) = validation.min {
            text = text.with_validator(move |input: &str| {
                if !input.trim().is_empty() && input.trim().len() < min {
                    Ok(Validation::Invalid(
                        format!("Must be at least {} characters", min).into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            });
        }
        if let Some(max) = validation.max {
            text = text.with_validator(move |input: &str| {
                if input.trim().len() > max {
                    Ok(Validation::Invalid(
                        format!(
                            "Must be at most {} characters (currently {})",
                            max,
                            input.trim().len()
                        )
                        .into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            });
        }

        if let Some(pattern_str) = &validation.pattern
            && let Ok(pattern) = regex::Regex::new(pattern_str)
        {
            text = text.with_validator(move |input: &str| {
                if !input.trim().is_empty() && !pattern.is_match(input) {
                    Ok(Validation::Invalid("Does not match required format".into()))
                } else {
                    Ok(Validation::Valid)
                }
            });
        }
    }
    let answer = text.prompt()?;

    if answer.trim().is_empty() && !field.required {
        Ok(String::new())
    } else {
        Ok(answer.trim().to_string())
    }
}

/// Prompt for confirm field
fn prompt_confirm(field: &Field) -> Result<String, InquireError> {
    let mut confirm = inquire::Confirm::new(&field.prompt);

    if let Some(help) = &field.help {
        confirm = confirm.with_help_message(help);
    }
    let answer = confirm.prompt()?;
    Ok(if answer {
        "yes".to_string()
    } else {
        "no".to_string()
    })
}
