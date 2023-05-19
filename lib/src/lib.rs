mod config;
mod errors;
mod exec;

pub use config::{Config, FailedTest, TaskInfo, TestResult};
pub use errors::{Error, Result};
pub use exec::CommandOutput;

pub type TaskID = String;
