use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod config;
mod git;
mod ui;

fn main() -> Result<()> {
    cli::Cli::parse().run()
}
