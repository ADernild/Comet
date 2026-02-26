use super::repository::find_repository;
use super::status::has_staged_changes;
use anyhow::{Context, Result, bail};
use git2::{Oid, Repository, Signature};
use std::io::Write;
use std::process::{Command, Stdio};

/// Get the default signature for commits (reads from git config)
fn get_signature<'a>(repo: &'a Repository) -> Result<Signature<'a>> {
    // Try to get signature from git config (user.name and user.email)
    if let Ok(sig) = repo.signature() {
        return Ok(sig);
    }

    // Fallback to default if not configured
    Signature::now("Unknown", "unknown@example.com").context("Failed to create signature")
}

/// Check if GPG signing is enabled in git config
fn is_signing_enabled(repo: &Repository) -> Result<bool> {
    let config = repo.config()?;

    // Check commit.gpgsign
    Ok(config.get_bool("commit.gpgsign").unwrap_or(false))
}

/// Get the GPG signing key from git config
fn get_signing_key(repo: &Repository) -> Result<Option<String>> {
    let config = repo.config()?;

    // Get user.signingkey
    match config.get_string("user.signingkey") {
        Ok(key) => Ok(Some(key)),
        Err(_) => Ok(None),
    }
}

/// Sign a commit buffer using GPG
fn sign_commit_buffer(commit_buf: &str, signing_key: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("gpg");

    cmd.arg("--detach-sign").arg("--armor");

    // If a specific key is provided, use it
    if let Some(key) = signing_key {
        cmd.arg("--local-user").arg(key);
    }

    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .context("Failed to spawn gpg process. Is GPG installed?")?;

    // Write commit buffer to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(commit_buf.as_bytes())
            .context("Failed to write to gpg stdin")?;
    }

    // Get the output
    let output = child
        .wait_with_output()
        .context("Failed to read gpg output")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("GPG signing failed: {}", stderr);
    }

    String::from_utf8(output.stdout).context("GPG signature is not valid UTF-8")
}

/// Create a commit with the given message
pub fn create_commit(message: &str) -> Result<Oid> {
    let repo = find_repository()?;

    // Check if there are staged changes
    if !has_staged_changes()? {
        bail!("No staged changes to commit. Use 'git add' to stage files.");
    }

    // Get the signature (author/committer info from git config)
    let signature = get_signature(&repo)?;

    // Get the current HEAD
    let head = repo.head().context("Failed to get HEAD reference")?;
    let parent_commit = head
        .peel_to_commit()
        .context("Failed to get parent commit")?;

    // Get the index and write it to a tree
    let mut index = repo.index().context("Failed to get repository index")?;
    let tree_oid = index.write_tree().context("Failed to write tree")?;
    let tree = repo.find_tree(tree_oid).context("Failed to find tree")?;

    // Check if signing is enabled
    let should_sign = is_signing_enabled(&repo)?;

    if should_sign {
        // Create signed commit
        let signing_key = get_signing_key(&repo)?;
        let parents: Vec<&git2::Commit> = vec![&parent_commit];

        // Create the commit buffer
        let commit_buf = repo.commit_create_buffer(
            &signature, // Author
            &signature, // Committer
            message,    // Commit message
            &tree,      // Tree
            &parents,   // Parents
        )?;

        // Convert buffer to string
        let commit_buf_str = commit_buf
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Commit buffer is not valid UTF-8"))?;

        // Sign the commit
        let signature_str = sign_commit_buffer(commit_buf_str, signing_key.as_deref())?;

        // Create the signed commit
        let commit_oid = repo
            .commit_signed(commit_buf_str, &signature_str, Some("gpgsig"))
            .context("Failed to create signed commit")?;

        // Update HEAD to point to the new commit
        let mut head_ref = repo.head().context("Failed to get HEAD reference")?;
        head_ref
            .set_target(commit_oid, "commit (signed)")
            .context("Failed to update HEAD")?;

        Ok(commit_oid)
    } else {
        // Create unsigned commit (original behavior)
        let commit_oid = repo
            .commit(
                Some("HEAD"),      // Update HEAD
                &signature,        // Author
                &signature,        // Committer
                message,           // Commit message
                &tree,             // Tree
                &[&parent_commit], // Parents
            )
            .context("Failed to create commit")?;

        Ok(commit_oid)
    }
}
