use anyhow::Result;
use inquire::Select;

use crate::{
    cli::InitArgs,
    config::{self, Template},
};

pub fn run(args: &InitArgs) -> Result<()> {
    let template = if args.conventional {
        Template::ConventionalCommits
    } else if args.minimal {
        Template::Minimal
    } else {
        // Get all available templates
        let templates: Vec<Template> = Template::all().collect();

        Select::new("Choose a commit template:", templates)
            .with_help_message("Select which format to use for your commits")
            .with_formatter(&|option| {
                let t = option.value;
                format!("{} - {}", t.name(), t.description())
            })
            .prompt()
            .map_err(|e| anyhow::anyhow!("Selection cancelled: {}", e))?
    };

    let config = template.build();
    let config_path = config::save(&config)?;

    println!("Created config file: {}", config_path.display());
    println!("  Template: {}", template.name());
    println!("\nYou can now customize your commit message format by editing this file.");

    Ok(())
}
