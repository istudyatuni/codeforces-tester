use std::io::Error as IOError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("empty command")]
    EmptyCommand,
    #[error("command exited with code {0}")]
    CommandExited(i32),
    #[error("command terminated")]
    CommandTerminated,

    #[error("cannot create command \"{0}\": {1}")]
    CannotCreateCommand(String, IOError),
    #[error("cannot save config: {0}")]
    CannotSaveConfig(IOError),
    #[error("cannot write to stdin: {0}")]
    CannotWriteToStdin(IOError),
    #[error("cannot read from stdout: {0}")]
    CannotReadFromStdout(IOError),
    #[error("cannot read from stderr: {0}")]
    CannotReadFromStderr(IOError),
    #[error("cannot get current directory: {0}")]
    CannotGetCwd(IOError),

    #[error("task \"{0}\" not found")]
    TaskNotFound(String),

    #[error("error serializing toml: {0}")]
    TomlSerialization(#[from] toml::ser::Error),
}
