use crate::{config, git, ui::prompt_commit_from_config};
use anyhow::Result;

pub fn run() -> Result<()> {
    // Check if we're in a git repository
    let _ = git::find_repository()?;

    // Checjk if there are staged changes
    if !git::has_staged_changes()? {
        println!("No staged changes to commit.");
        println!("Use 'git add <files>' to stage changes first.");
        return Ok(());
    }

    // Show what's staged
    let staged_files = git::get_staged_files()?;
    if !staged_files.is_empty() {
        println!("Staged files:");
        for file in &staged_files {
            println!("   • {}", file);
        }
        println!();
    }

    let current_branch = git::get_current_branch()?;
    println!("Branch: {}", current_branch);
    println!();

    // Load config (from file or default)
    let config = config::load()?;

    let commit_message =
        prompt_commit_from_config(&config).map_err(|e| anyhow::anyhow!("Prompt error {}", e))?;

    println!("\nCommit message:\n");
    println!("─────────────────────────────────────");
    println!("{}", commit_message);
    println!("─────────────────────────────────────");

    // Confirm before committing
    let confirm = inquire::Confirm::new("Create this commit?")
        .with_default(true)
        .prompt()
        .map_err(|e| anyhow::anyhow!("Prompt error: {}", e))?;

    if !confirm {
        println!("❌ Commit cancelled");
        return Ok(());
    }

    // Create the commit
    let commit_oid = git::create_commit(&commit_message)?;

    println!("Commit created: {}", commit_oid);

    Ok(())
}
