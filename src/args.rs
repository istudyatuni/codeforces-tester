use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Args {
    #[cfg(feature = "gui")]
    #[arg(long)]
    /// Run gui
    pub(crate) gui: bool,

    #[cfg(feature = "cli")]
    #[command(flatten)]
    cli: cli::Cli,
}

impl Args {
    #[cfg(feature = "cli")]
    pub(crate) fn cli(&self) -> &cli::Cli {
        &self.cli
    }
}
