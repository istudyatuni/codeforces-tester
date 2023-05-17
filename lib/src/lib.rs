mod config;
mod errors;
mod exec;

pub use config::Config;

pub use errors::{Error, Result};

pub type TaskID = String;
