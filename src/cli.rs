use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands;

#[derive(Parser)]
#[command(
    name = "git-cmt",
    version,
    about = "Comet - structured commit messages made easy",
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        self.command
            .as_ref()
            .unwrap_or(&Commands::Commit {})
            .execute()
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Commit changes interactively
    Commit {},

    /// Initialize comet config
    Init {},

    /// Generate changelog from commits
    Changelog {},

    /// Show commit stats
    Stats {},
}

impl Commands {
    pub fn execute(&self) -> Result<()> {
        match self {
            Commands::Commit {} => commands::commit::run()?,
            Commands::Init {} => commands::init::run()?,
            Commands::Changelog {} => commands::changelog::run()?,
            Commands::Stats {} => commands::stats::run()?,
        }
        Ok(())
    }
}
