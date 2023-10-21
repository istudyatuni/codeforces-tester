use std::path::PathBuf;

use clap::{Args, Subcommand};
use lib::TaskID;

#[derive(Debug, Args)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(long, default_value = "cdf.toml")]
    /// Path to config
    config: PathBuf,

    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

impl Cli {
    pub(crate) fn config(&self) -> &PathBuf {
        &self.config
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Add a new task
    Add,
    /// Run test for specific task
    Test {
        /// Test id
        id: TaskID,
    },
    /// Format config file
    #[clap(name = "fmt")]
    Format,
    /// Initialize default config in current directory
    #[clap(name = "init")]
    Init {
        /// Use sample config instead of minimal
        #[arg(long)]
        sample: bool,
    },
}
