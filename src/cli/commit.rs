use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser, Default)]
pub struct CommitArgs {
    #[arg(short = 'f', long = "field", value_parser=parse_key_val, number_of_values = 1)]
    pub fields: Vec<(String, String)>,
    #[arg(long)]
    pub no_prompt: bool,
}

impl CommitArgs {
    pub fn to_values(&self) -> HashMap<String, String> {
        self.fields.iter().cloned().collect()
    }
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s.find('=').ok_or("must be in key=value format")?;

    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}
