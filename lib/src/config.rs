use std::{collections::BTreeMap, fs::write as write_file, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    exec::{exec, exec_with_io, CommandOutput},
    Error, Result, TaskID,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Debug)]
pub struct TaskInfo {
    pub id: TaskID,
    pub name: String,
    pub tests_count: usize,
}

impl TaskInfo {
    pub fn new<S: Into<String>>(id: S, name: S, tests_count: usize) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            tests_count,
        }
    }
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
    pub fn check_task(&self, id: &TaskID) -> Result<()> {
        let Some(task) = self.tasks.get(id) else {
            return Err(crate::Error::TaskNotFound(id.clone()));
        };
        if task.tests.is_empty() {
            return Err(Error::TaskHasNoTests(id.clone()));
        }
        Ok(())
    }
    pub fn should_build(&self, _id: &TaskID) -> bool {
        self.settings.build.build.is_some()
    }
    pub fn build(&self, id: &TaskID) -> Result<()> {
        if let Some(build) = &self.settings.build.build {
            exec(build.replace("{id}", id), &self.settings.build.cwd)?;
        }
        Ok(())
    }
    pub fn run_tests<'s>(&'s self, id: &'s TaskID) -> impl IntoIterator<Item = TestResult> + 's {
        let tests = self
            .tasks
            .get(id)
            .map(|t| t.tests.clone())
            .unwrap_or_default();
        tests.into_iter().enumerate().map(|(i, test)| {
            let output = exec_with_io(
                self.settings.build.run.replace("{id}", id),
                test.input,
                &self.settings.build.cwd,
            );
            let output = match output {
                Ok(c) => c,
                Err(e) => return TestResult::Err(e),
            };
            if output.stdout.trim() != test.expected.trim() {
                TestResult::Failed(FailedTest::new(i, test.expected, output))
            } else {
                TestResult::Ok
            }
        })
    }
    pub fn get_task_name(&self, id: &TaskID) -> Option<String> {
        self.tasks.get(id).map(|t| t.name.clone())
    }
    pub fn add_task<S: Into<String>>(&mut self, id: TaskID, name: S) {
        self.tasks.entry(id.to_lowercase()).or_default().name = name.into()
    }
    pub fn add_test_to_task<S>(&mut self, id: TaskID, input: S, expected: S)
    where
        S: Into<String>,
    {
        self.tasks
            .entry(id)
            .or_default()
            .tests
            .push(Test::new(input, expected))
    }
    pub fn save_config_to(&self, path: PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        write_file(path, content.as_bytes()).map_err(Error::CannotSaveConfig)
    }
    pub fn tasks(&self) -> impl Iterator<Item = TaskInfo> + '_ {
        self.tasks
            .iter()
            .map(|(k, v)| TaskInfo::new(k, &v.name, v.tests.len()))
    }
}

#[derive(Debug)]
pub enum TestResult {
    Ok,
    Failed(FailedTest),
    Err(Error),
}

#[derive(Debug)]
pub struct FailedTest {
    pub index: usize,
    pub expected: String,
    pub cmd_output: CommandOutput,
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
