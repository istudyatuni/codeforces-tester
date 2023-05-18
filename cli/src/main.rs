use std::fs::read_to_string;

use anyhow::Result;
use clap::Parser;

use args::{Cli, Commands};
use input::{read_line_with_prompt, read_until_eof_with_prompt};
use lib::{Config, FailedTest, TaskID, TestResult};

mod args;
mod input;

#[cfg(target_family = "unix")]
const EOF_KEYBOARD: &str = "Ctrl+D";
#[cfg(target_family = "windows")]
const EOF_KEYBOARD: &str = "Ctrl+Z";

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = cli.config().clone();
    config_path.try_exists()?;
    let config = read_to_string(&config_path)?;
    let mut config = Config::try_from(config.as_str())?;

    let Some(command) = cli.command else {
        return Ok(());
    };
    match command {
        Commands::Add => {
            ask_and_add_task(&mut config)?;
            config.save_config_to(config_path.clone())?;
            println!("Saved to {}", config_path.display());
        }
        Commands::Test { id } => run_task_tests(&config, id)?,
        Commands::Format => config.save_config_to(config_path)?,
    }

    Ok(())
}

fn ask_and_add_task(config: &mut Config) -> Result<()> {
    let id = read_line_with_prompt("Enter task_id: ")?;
    let id = id.trim().into();
    let name = match config.get_task_name(&id) {
        Some(n) => n,
        None => read_line_with_prompt("Enter task name: ")?,
    };
    let continue_prompt = format!("(press {EOF_KEYBOARD} to continue)");
    let prompt = format!("Enter task input {continue_prompt}:\n");
    let input = read_until_eof_with_prompt(&prompt)?;

    let prompt = format!("Enter expected output {continue_prompt}:\n");
    let expected = read_until_eof_with_prompt(&prompt)?;
    config.add_test_to_task(id, name, input, expected);
    Ok(())
}

fn run_task_tests(config: &Config, id: TaskID) -> Result<()> {
    config.check_task(&id)?;
    println!(
        "Task {} - {}",
        id.to_uppercase(),
        config.get_task_name(&id).unwrap_or("unnamed task".into())
    );
    if config.shold_build(&id) {
        println!("Building");
        config.build(&id)?;
    }
    println!("Testing");
    let mut failed = vec![];
    for res in config.tests(&id) {
        match res as TestResult {
            TestResult::Ok => print!("."),
            TestResult::Failed(f) => {
                print!("x");
                failed.push(f);
            }
            TestResult::Err(e) => return Err(e.into()),
        }
    }
    if !failed.is_empty() {
        println!(" failed\n");
        failed.iter().for_each(print_failed_test);
    } else {
        println!(" ok");
    }

    Ok(())
}

fn print_failed_test(f: &FailedTest) {
    let mut stderr = String::new();
    if !f.cmd_output.stderr.is_empty() {
        stderr = format!("\nStderr:\n{}", f.cmd_output.stderr);
    }
    println!(
        "-- test {} --\nExpected output:\n{}\n\nActual output:\n{}{stderr}",
        f.index + 1, f.expected, f.cmd_output.stdout
    );
}
