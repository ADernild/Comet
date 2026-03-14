mod commit;
mod init;
mod stats;

use anyhow::Result;
use clap::{Parser, Subcommand};

pub use commit::CommitArgs;
pub use init::InitArgs;

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
    Init(InitArgs),
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
            Commands::Init(args) => commands::init::run(args)?,
        }
        Ok(())
    }
}
