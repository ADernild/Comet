use crate::config::schema::*;
use std::collections::HashMap;

fn field(id: &str, required: bool, field_type: FieldType) -> Field {
    Field {
        id: id.to_string(),
        field_type,
        prompt: format!("{} field", id),
        required,
        help: None,
        options: None,
        validate: None,
        wrap: None,
        values: None,
    }
}

fn config(template: &str, fields: Vec<Field>) -> Config {
    Config {
        output: OutputConfig {
            template: template.to_string(),
        },
        fields,
        // rules: vec![],
    }
}

fn values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// ── Validation ───────────────────────────────────────────────────

#[test]
fn test_validate_rejects_undefined_placeholders() {
    let c = config(
        "{type}: {undefined_field}",
        vec![field("type", true, FieldType::Text)],
    );
    assert!(c.validate().is_err());
}

#[test]
fn test_validate_rejects_unused_fields() {
    let c = config(
        "{type}: {description}",
        vec![
            field("type", true, FieldType::Text),
            field("description", true, FieldType::Text),
            field("unused_field", false, FieldType::Text),
        ],
    );
    assert!(c.validate().is_err());
}

#[test]
fn test_validate_accepts_valid_config() {
    let c = config(
        "{type}({scope}): {description}",
        vec![
            field("type", true, FieldType::Text),
            field("scope", false, FieldType::Text),
            field("description", true, FieldType::Text),
        ],
    );
    assert!(c.validate().is_ok());
}

// ── Render ───────────────────────────────────────────────────────

#[test]
fn test_render_all_fields_present() {
    let c = config(
        "{type}({scope}): {description}",
        vec![
            field("type", true, FieldType::Text),
            field("scope", false, FieldType::Text),
            field("description", true, FieldType::Text),
        ],
    );
    let result = c
        .render(&values(&[
            ("type", "feat"),
            ("scope", "api"),
            ("description", "add endpoint"),
        ]))
        .unwrap();
    assert_eq!(result, "feat(api): add endpoint");
}

#[test]
fn test_render_removes_empty_optional_field() {
    let c = config(
        "{type}({scope}): {description}",
        vec![
            field("type", true, FieldType::Text),
            field("scope", false, FieldType::Text),
            field("description", true, FieldType::Text),
        ],
    );
    let result = c
        .render(&values(&[
            ("type", "feat"),
            ("scope", ""),
            ("description", "add endpoint"),
        ]))
        .unwrap();
    assert_eq!(result, "feat: add endpoint");
}

#[test]
fn test_render_multiline_fields() {
    let c = config(
        "{type}: {description}\n\n{body}\n\n{footer}",
        vec![
            field("type", true, FieldType::Text),
            field("description", true, FieldType::Text),
            field("body", false, FieldType::Multiline),
            field("footer", false, FieldType::Text),
        ],
    );
    let result = c
        .render(&values(&[
            ("type", "feat"),
            ("description", "add feature"),
            ("body", "Detailed explanation"),
            ("footer", "Closes #123"),
        ]))
        .unwrap();
    assert_eq!(
        result,
        "feat: add feature\n\nDetailed explanation\n\nCloses #123"
    );
}

#[test]
fn test_render_removes_empty_optional_multiline_fields() {
    let c = config(
        "{type}: {description}\n\n{body}\n\n{footer}",
        vec![
            field("type", true, FieldType::Text),
            field("description", true, FieldType::Text),
            field("body", false, FieldType::Multiline),
            field("footer", false, FieldType::Text),
        ],
    );
    let result = c
        .render(&values(&[
            ("type", "feat"),
            ("description", "add feature"),
            ("body", ""),
            ("footer", ""),
        ]))
        .unwrap();
    assert_eq!(result, "feat: add feature");
}

#[test]
fn test_render_rejects_empty_required_fields() {
    let c = config(
        "{type}({scope}): {description}",
        vec![
            field("type", true, FieldType::Select),
            field("scope", false, FieldType::Multiline),
            field("description", true, FieldType::Text),
        ],
    );
    assert!(
        c.render(&values(&[
            ("type", ""),
            ("scope", ""),
            ("description", "add feature")
        ]))
        .is_err()
    );
}
