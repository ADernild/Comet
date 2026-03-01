use crate::config::schema::{Config, Field, FieldType, OutputConfig, Validation};

pub fn minimal() -> Config {
    Config {
        output: OutputConfig {
            template: "{type}: {description}".to_string(),
        },
        fields: vec![
            Field {
                id: "type".to_string(),
                field_type: FieldType::Select,
                prompt: "Commit type".to_string(),
                required: true,
                help: Some("Type of change".to_string()),
                options: Some(vec![
                    "feat".to_string(),
                    "fix".to_string(),
                    "docs".to_string(),
                    "refactor".to_string(),
                    "test".to_string(),
                    "chore".to_string(),
                ]),
                validate: None,
                wrap: None,
                values: None,
            },
            Field {
                id: "description".to_string(),
                field_type: FieldType::Text,
                prompt: "Description".to_string(),
                required: true,
                help: Some("Brief description of changes".to_string()),
                options: None,
                validate: Some(Validation {
                    min: Some(1),
                    max: Some(72),
                    pattern: None,
                }),
                wrap: None,
                values: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_config_is_valid() {
        let config = minimal();
        assert!(config.validate().is_ok());
    }
}
