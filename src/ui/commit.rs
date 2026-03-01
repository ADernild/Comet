use anyhow::Result;
use inquire::{InquireError, Select, Text, required, validator::Validation};

use crate::config::{Field, FieldType};

/// Prompt for a single field based on its configuration
pub fn prompt_field(field: &Field) -> Result<String, InquireError> {
    match field.field_type {
        FieldType::Select => prompt_select(field),
        FieldType::Text | FieldType::Multiline => prompt_text(field),
        FieldType::Confirm => prompt_confirm(field),
    }
}

/// Prompt for select field
fn prompt_select(field: &Field) -> Result<String, InquireError> {
    let options = field.options.as_ref().unwrap();

    let mut select = Select::new(&field.prompt, options.clone());

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

/// Prompt for boolean confirmation
pub fn confirm(message: &str, help: Option<&str>, default: bool) -> Result<bool, InquireError> {
    let mut confirm = inquire::Confirm::new(message).with_default(default);

    if let Some(_help) = help {
        confirm = confirm.with_help_message(_help);
    }

    confirm.prompt()
}

/// Prompt for confirm field
fn prompt_confirm(field: &Field) -> Result<String, InquireError> {
    let answer = confirm(&field.prompt, field.help.as_deref(), false)?;

    // Use custom values if provided, otherwise default to "yes"/"no"
    Ok(if let Some(values) = &field.values {
        if answer {
            values.on_true.clone()
        } else {
            values.on_false.clone()
        }
    } else {
        if answer {
            "yes".to_string()
        } else {
            "no".to_string()
        }
    })
}
