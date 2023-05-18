use std::{collections::BTreeMap, fs::write as write_file, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    exec::{exec, exec_with_io, CommandOutput},
    Error, Result, TaskID,
};

#[derive(Debug, Deserialize, Serialize)]
struct Test {
    input: String,
    expected: String,
}

impl Test {
    fn new<S: Into<String>>(input: S, expected: S) -> Self {
        Self {
            input: input.into(),
            expected: expected.into(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Task {
    name: String,
    tests: Vec<Test>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Settings {
    build: BuildSettings,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    settings: Settings,
    tasks: BTreeMap<TaskID, Task>,
}

impl TryFrom<&str> for Config {
    type Error = toml::de::Error;

    fn try_from(value: &str) -> Result<Config, Self::Error> {
        toml::from_str(value)
    }
}

impl Config {
    pub fn run_task_tests(&self, id: TaskID) -> Result<()> {
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
    pub fn get_task_name(&self, id: &TaskID) -> Option<String> {
        self.tasks.get(id).map(|t| t.name.clone())
    }
    pub fn add_test_to_task<S>(&mut self, id: TaskID, name: S, input: S, expected: S)
    where
        S: Into<String>,
    {
        let task = self.tasks.entry(id).or_default();
        task.name = name.into();
        task.tests.push(Test::new(input, expected));
    }
    pub fn save_config_to(&self, path: PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        write_file(path, content.as_bytes()).map_err(Error::CannotSaveConfig)
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
