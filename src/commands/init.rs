use anyhow::Result;

use crate::config;

pub fn run() -> Result<()> {
    let config = config::conventional_commits();
    let config_path = config::save(&config)?;

    println!("Created config file: {}", config_path.display());
    println!("\nYou can now customize your commit message format by editing this file.");

    Ok(())
}
