use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};

use super::schema::Config;

impl Config {
    /// Render the commit message by substituting field values into the template.
    ///
    /// Optional fields that are emtpy are removed (along with empty parantheses, etc.).
    /// Required fields that are empty produce an error.
    pub fn render(&self, values: &HashMap<String, String>) -> Result<String> {
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
        Ok(Self::clean_output(&output))
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
}
