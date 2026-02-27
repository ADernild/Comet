// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Alexander Dernild

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
