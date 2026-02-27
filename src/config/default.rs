use super::schema::{Config, Field, FieldType, OutputConfig, Validation};

/// Default conventional commits configuration
pub fn conventional_commits() -> Config {
    Config {
        output: OutputConfig {
            template: "{type}({scope}): {description}\n\n{body}\n\n{footer}".to_string(),
        },
        fields: vec![
            Field {
                id: "type".to_string(),
                field_type: FieldType::Select,
                prompt: "Select commit type".to_string(),
                required: true,
                help: Some("Type of change you're committing".to_string()),
                options: vec![
                    "feat".to_string(),
                    "fix".to_string(),
                    "docs".to_string(),
                    "style".to_string(),
                    "refactor".to_string(),
                    "perf".to_string(),
                    "test".to_string(),
                    "build".to_string(),
                    "ci".to_string(),
                    "chore".to_string(),
                    "revert".to_string(),
                ],
                validate: None,
                wrap: None,
            },
            Field {
                id: "scope".to_string(),
                field_type: FieldType::Text,
                prompt: "Scope (optional)".to_string(),
                required: false,
                help: Some("Component affected (e.g., api, auth, ui)".to_string()),
                options: vec![],
                validate: Some(Validation {
                    min: Some(1),
                    max: Some(20),
                    pattern: None,
                }),
                wrap: None,
            },
            Field {
                id: "description".to_string(),
                field_type: FieldType::Text,
                prompt: "Description".to_string(),
                required: true,
                help: Some("Brief description of changes (1-72 characters)".to_string()),
                options: vec![],
                validate: Some(Validation {
                    min: Some(1),
                    max: Some(72),
                    pattern: None,
                }),
                wrap: None,
            },
            Field {
                id: "body".to_string(),
                field_type: FieldType::Multiline,
                prompt: "Body (optional)".to_string(),
                required: false,
                help: Some("Detailed explanation of changes".to_string()),
                options: vec![],
                validate: None,
                wrap: Some(72),
            },
            Field {
                id: "footer".to_string(),
                field_type: FieldType::Text,
                prompt: "Footer (optional)".to_string(),
                required: false,
                help: Some("Breaking changes, issue references (e.g., 'Closes #42'".to_string()),
                options: vec![],
                validate: Some(Validation {
                    min: None,
                    max: None,
                    pattern: Some(r"^[a-zA-Z-]+[: #].+$".to_string()),
                }),
                wrap: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conventional_commits_config_is_valid() {
        let config = conventional_commits();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_conventional_commits_has_expected_fields() {
        let config = conventional_commits();

        assert_eq!(config.fields.len(), 5);
        assert_eq!(config.fields[0].id, "type");
        assert_eq!(config.fields[1].id, "scope");
        assert_eq!(config.fields[2].id, "description");
        assert_eq!(config.fields[3].id, "body");
        assert_eq!(config.fields[4].id, "footer");
    }

    #[test]
    fn test_conventional_commits_type_field_is_select() {
        let config = conventional_commits();
        let type_field = &config.fields[0];

        assert!(matches!(type_field.field_type, FieldType::Select));
        assert!(type_field.required);
        assert!(!type_field.options.is_empty());
    }

    #[test]
    fn test_conventional_commits_required_fields() {
        let config = conventional_commits();

        // Type and description should be required
        assert!(
            config
                .fields
                .iter()
                .find(|f| f.id == "type")
                .unwrap()
                .required
        );
        assert!(
            config
                .fields
                .iter()
                .find(|f| f.id == "description")
                .unwrap()
                .required
        );

        // Scope, body, footer should be optional
        assert!(
            !config
                .fields
                .iter()
                .find(|f| f.id == "scope")
                .unwrap()
                .required
        );
        assert!(
            !config
                .fields
                .iter()
                .find(|f| f.id == "body")
                .unwrap()
                .required
        );
        assert!(
            !config
                .fields
                .iter()
                .find(|f| f.id == "footer")
                .unwrap()
                .required
        );
    }
}
