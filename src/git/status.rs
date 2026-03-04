use anyhow::{Context, Result};

use super::repository::find_repository;

/// Check if there are staged changes
pub fn has_staged_changes() -> Result<bool> {
    let repo = find_repository()?;
    let mut index = repo.index().context("Failed to get repository index")?;

    // Get the current tree (HEAD)
    let head_tree = match repo.head() {
        Ok(head) => Some(head.peel_to_tree().context("Failed to get HEAD tree")?),
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => None,
        Err(e) => return Err(e).context("Failed to get HEAD reference"),
    };

    // let head = repo.head().context("Failed to get HEAD reference")?;
    // let head_tree = head.peel_to_tree().context("Failed to get HEAD tree")?;

    // Get the index tree
    let index_oid = index.write_tree().context("Failed to write index tree")?;
    let index_tree = repo
        .find_tree(index_oid)
        .context("Failed to find index tree")?;

    let head_tree = match head_tree {
        Some(tree) => tree,
        None => return Ok(!index.is_empty()),
    };

    // Compare trees
    Ok(head_tree.id() != index_tree.id())
}

/// Get list of staged files
pub fn get_staged_files() -> Result<Vec<String>> {
    let repo = find_repository()?;

    let index = repo.index().context("Failed to get repository index")?;

    // Handle unborn branch (no commits yet)
    let tree = match repo.head() {
        Ok(head) => Some(head.peel_to_tree().context("Failed to get HEAD tree")?),
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => None,
        Err(e) => return Err(e).context("Failed to get HEAD reference"),
    };

    let diff = match tree {
        Some(tree) => {
            // Compare HEAD to index
            repo.diff_tree_to_index(Some(&tree), Some(&index), None)
                .context("Failed to get diff")?
        }
        None => {
            // No HEAD yet, compare empty tree to index (all staged files are new)
            let empty_tree = repo.find_tree(repo.treebuilder(None)?.write()?)?;
            repo.diff_tree_to_index(Some(&empty_tree), Some(&index), None)
                .context("Failed to get diff")?
        }
    };

    let mut files = Vec::new();

    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path()
                && let Some(path_str) = path.to_str()
            {
                files.push(path_str.to_string());
            }
            true
        },
        None,
        None,
        None,
    )?;

    Ok(files)
}
