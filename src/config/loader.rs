use super::schema::Config;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Find config file in git repository root
fn find_config_file() -> Result<Option<PathBuf>> {
    // Try to find config in git root
    if let Ok(git_root) = crate::git::get_repo_root() {
        let config_path = git_root.join(".comet.toml");
        if config_path.exists() {
            return Ok(Some(config_path));
        }
    }

    // If not in git repo or no config found, return None (will use default)
    Ok(None)
}

/// Load config from a file path
pub fn load_from_file(path: &Path) -> Result<Config> {
    let contents = std::fs::read_to_string(path).context("Failed to parse config file")?;

    let config: Config = toml::from_str(&contents).context("Failed to parse config file")?;

    Ok(config)
}

/// Save config to git repository root
pub fn save(config: &Config) -> Result<PathBuf> {
    let git_root =
        crate::git::get_repo_root().context("Not in a git repository. Cannot save config.")?;

    let config_path = git_root.join(".comet.toml");

    if config_path.exists() {
        anyhow::bail!("Config file already exists: {}", config_path.display());
    }

    let toml_string = toml::to_string_pretty(config).context("Failed to serialize config")?;

    std::fs::write(&config_path, toml_string).context(format!(
        "Failed to write config file: {}",
        config_path.display()
    ))?;

    Ok(config_path)
}

/// Load config from project root, or use default
pub fn load() -> Result<Config> {
    let config_path = find_config_file()?;

    if let Some(path) = config_path {
        load_from_file(&path)
    } else {
        Ok(super::default::conventional_commits())
    }
}
