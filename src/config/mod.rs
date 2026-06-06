mod loader;
mod render;
mod schema;
mod templates;

pub use loader::{load, save};
pub use schema::{Config, Field, FieldType};
pub use templates::Template;

#[cfg(test)]
mod tests;
