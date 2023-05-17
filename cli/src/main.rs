use std::fs::read_to_string;

use anyhow::Result;
use clap::Parser;

use args::Cli;
use lib::Config;

mod args;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = cli.config();
    config.try_exists()?;
    let config = read_to_string(config)?;
    let config = Config::try_from(config.as_str())?;

    if let Some(test_id) = cli.test_id() {
        config.run_task_tests(test_id)?;
    }
    Ok(())
}
