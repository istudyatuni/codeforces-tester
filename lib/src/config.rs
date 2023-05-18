#![allow(unused)]

use std::{collections::HashMap, path::PathBuf, process::Command};

use serde::Deserialize;

use crate::{
    exec::{exec, exec_with_io, CommandOutput},
    Result, TaskID,
};

#[derive(Debug, Deserialize)]
pub(crate) struct Test {
    input: String,
    expected: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Task {
    name: String,
    tests: Vec<Test>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Settings {
    build: BuildSettings,
}

#[derive(Debug, Deserialize)]
/// Available placeholders:
/// - `{id}` - task id
struct BuildSettings {
    /// Build command (optional)
    build: Option<String>,
    /// Run command
    run: String,
    /// Working directory for executing commands, can be absolute or relative
    cwd: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    settings: Settings,
    tasks: HashMap<TaskID, Task>,
}

impl TryFrom<&str> for Config {
    type Error = toml::de::Error;

    fn try_from(value: &str) -> Result<Config, Self::Error> {
        toml::from_str(value)
    }
}

impl Config {
    pub fn run_task_tests(&self, id: String) -> Result<()> {
        let Some(task) = self.tasks.get(&id) else {
            return Err(crate::Error::TaskNotFound(id));
        };
        if task.tests.is_empty() {
            println!("No tests for task");
            return Ok(());
        }

        println!("Task {} - {}", id.to_uppercase(), task.name);

        // build
        if let Some(build) = &self.settings.build.build {
            println!("Building");
            exec(build.replace("{id}", &id), &self.settings.build.cwd)?;
        }

        // test
        println!("Testing");
        let mut failed = vec![];
        for (i, test) in task.tests.iter().enumerate() {
            let output = exec_with_io(
                self.settings.build.run.replace("{id}", &id),
                test.input.clone(),
                &self.settings.build.cwd,
            )?;
            if output.stdout.trim() != test.expected.trim() {
                failed.push(FailedTest::new(i + 1, &test.expected, output));
                print!("x");
            } else {
                print!(".");
            };
        }
        if failed.is_empty() {
            println!(" ok");
            return Ok(());
        }

        println!("\n\nFailed tests:\n");
        for f in failed {
            println!(
                "-- test {} --\nExpected output:\n{}\n\nActual output:\n{}",
                f.index, f.expected, f.cmd_output.stdout
            );
            if !f.cmd_output.stderr.is_empty() {
                println!("Stderr:\n{}", f.cmd_output.stderr);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct FailedTest {
    index: usize,
    expected: String,
    cmd_output: CommandOutput,
}

impl FailedTest {
    fn new<S: Into<String>>(index: usize, expected: S, cmd_output: CommandOutput) -> Self {
        Self {
            index,
            expected: expected.into(),
            cmd_output,
        }
    }
}
