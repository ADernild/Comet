use std::path::PathBuf;

use anyhow::{Context, Result};
use git2::Repository;

/// Find the git repository starting from current directory
pub fn find_repository() -> Result<Repository> {
    Repository::discover(".")
        .context("Not in a git repository. Please run this command from within a git repository.")
}

/// Get the root directory of the git repository
pub fn get_repo_root() -> Result<PathBuf> {
    let repo = find_repository()?;

    repo.workdir()
        .map(|p| p.to_path_buf())
        .context("Repository has no working directory")
}

/// Get the current branch name
pub fn get_current_branch() -> Result<String> {
    let repo = find_repository()?;

    match repo.head() {
        Ok(head) => Ok(head.shorthand().unwrap_or("HEAD").to_string()),
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
            // No commits yet - return the default branch name
            Ok("main (no commits yet)".to_string())
        }
        Err(e) => Err(e).context("Failed to get HEAD reference"),
    }
}
