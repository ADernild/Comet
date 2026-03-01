use strum::{Display, EnumIter, EnumMessage, IntoEnumIterator};

mod conventional;
mod minimal;

use crate::config::Config;

pub use conventional::conventional_commits;
pub use minimal::minimal;

#[derive(Debug, Clone, Copy, EnumIter, EnumMessage, Display)]
pub enum Template {
    /// Full conventional commits spec (type, scope, description, body, footer)
    #[strum(serialize = "Conventional Commits")]
    ConventionalCommits,

    /// Lightweight format (type and description only)
    #[strum(serialize = "Minimal")]
    Minimal,
}

impl Template {
    pub fn build(self) -> Config {
        match self {
            Template::ConventionalCommits => conventional_commits(),
            Template::Minimal => minimal(),
        }
    }

    pub fn all() -> impl Iterator<Item = Template> {
        Template::iter()
    }

    pub fn name(&self) -> String {
        // Uses Display from strum
        self.to_string()
    }

    pub fn description(&self) -> &str {
        self.get_message().unwrap_or("")
    }
}
