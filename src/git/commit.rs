use std::io::Write;
use std::process::Command;

use anyhow::{Context, Result, bail};
use tempfile::NamedTempFile;

/// Create a commit with the given message by shelling out to git
pub fn create_commit(message: &str) -> Result<String> {
    let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;

    write!(temp_file, "{}", message).context("Failed to write commit message to temporary file")?;

    temp_file
        .flush()
        .context("Failed to flush temporary file")?;

    let status = Command::new("git")
        .arg("commit")
        .arg("-F")
        .arg(temp_file.path())
        .status()
        .context("Failed to execute git commit")?;

    if !status.success() {
        bail!("Git commit failed with status: {}", status);
    }

    let hash_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .context("Failed to get commit hash")?;

    if !hash_output.status.success() {
        bail!("Failed to retrieve commit hash");
    }

    let hash = String::from_utf8_lossy(&hash_output.stdout)
        .trim()
        .to_string();

    Ok(hash)
}
