use std::collections::HashMap;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands;

#[derive(Parser)]
#[command(
    name = "git-cmt",
    version,
    about = "Comet - structured commit messages made easy",
    author,
    args_conflicts_with_subcommands = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub commit_args: CommitArgs,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Commit changes interactively
    Commit(CommitArgs),

    /// Initialize comet config
    Init {},

    /// Generate changelog from commits
    Changelog {},

    /// Show commit stats
    Stats {},
}

#[derive(Debug, Parser, Default)]
pub struct CommitArgs {
    #[arg(short = 'f', long = "field", value_parser=parse_key_val, number_of_values = 1)]
    pub fields: Vec<(String, String)>,
    #[arg(long)]
    pub no_prompt: bool,
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Some(cmd) => cmd.execute(),
            None => commands::commit::run(&self.commit_args),
        }
    }
}

impl Commands {
    pub fn execute(&self) -> Result<()> {
        match self {
            Commands::Commit(args) => commands::commit::run(args)?,
            Commands::Init {} => commands::init::run()?,
            Commands::Changelog {} => commands::changelog::run()?,
            Commands::Stats {} => commands::stats::run()?,
        }
        Ok(())
    }
}

impl CommitArgs {
    pub fn to_values(&self) -> HashMap<String, String> {
        self.fields.iter().cloned().collect()
    }
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s.find('=').ok_or("must be in key=value format")?;

    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
