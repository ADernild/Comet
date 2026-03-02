use clap::Parser;

#[derive(Debug, Parser, Default)]
pub struct InitArgs {
    /// Use conventional commits template
    #[arg(long, conflicts_with = "minimal")]
    pub conventional: bool,

    /// Use minimal template
    #[arg(long, conflicts_with = "conventional")]
    pub minimal: bool,
}
