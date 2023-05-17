use std::path::PathBuf;

use clap::{Parser, Subcommand};
use lib::TaskID;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[arg(long, default_value = "cdf.toml")]
    /// Path to config. Default is "cdf.toml" in current directory
    config: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    pub(crate) fn config(&self) -> &PathBuf {
        &self.config
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Add new task (not implemented)
    Add,
    /// Run test by its id
    Test { id: TaskID },
}

impl Cli {
    pub(crate) fn test_id(&self) -> Option<TaskID> {
        if let Some(Commands::Test { id }) = &self.command {
            return Some(id.into());
        }
        None
    }
}
