use std::collections::HashMap;

use anyhow::Result;

use crate::{cli::CommitArgs, config, git, ui};

pub fn run(args: &CommitArgs) -> Result<()> {
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

    // Resolve field values from args and prompts
    let values = resolve_values(args, &config)?;

    let commit_message = config.render(&values)?;

    println!("\nCommit message:\n");
    println!("─────────────────────────────────────");
    println!("{}", commit_message);
    println!("─────────────────────────────────────");

    // Confirm before committing
    match ui::confirm("Create this commit?", None, true)? {
        true => {
            let commit_oid = git::create_commit(&commit_message)?;
            println!("Commit created: {}", commit_oid);
        }
        false => println!("Commit cancelled"),
    }

    Ok(())
}

fn resolve_values(args: &CommitArgs, config: &config::Config) -> Result<HashMap<String, String>> {
    let mut values = args.to_values();

    // If no_prompt is set, validate we have all required fields
    if args.no_prompt {
        let missing: Vec<&str> = config
            .fields
            .iter()
            .filter(|f| f.required && !values.contains_key(&f.id))
            .map(|f| f.id.as_str())
            .collect();

        if !missing.is_empty() {
            anyhow::bail!("Missing required fields: {}", missing.join(", "));
        }
    } else {
        // Prompt for missing fields
        for field in &config.fields {
            if !values.contains_key(&field.id) {
                let value = ui::prompt_field(field)?;
                values.insert(field.id.clone(), value);
            }
        }
    }

    // Apply wrapping to multiline fields
    for field in &config.fields {
        if let Some(value) = values.get_mut(&field.id)
            && matches!(field.field_type, config::FieldType::Multiline)
            && !value.is_empty()
            && let Some(width) = field.wrap
        {
            *value = textwrap::fill(value, width);
        }
    }

    Ok(values)
}
