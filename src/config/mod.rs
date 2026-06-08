mod loader;
mod render;
mod rules;
mod schema;
mod templates;

pub use loader::{load, save};
pub use rules::evaluate_rules;
pub use schema::{Config, Field, FieldType};
pub use templates::Template;

#[cfg(test)]
mod tests;
