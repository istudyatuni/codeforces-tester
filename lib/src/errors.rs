pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("empty command")]
    EmptyCommand,
    #[error("command exited with code {0}")]
    CommandExited(i32),
    #[error("command terminated")]
    CommandTerminated,

    #[error("cannot create command: {0}")]
    CannotCreateCommand(std::io::Error),
    #[error("cannot write to stdin: {0}")]
    CannotWriteToStdin(std::io::Error),
    #[error("cannot read from stdout: {0}")]
    CannotReadFromStdout(std::io::Error),
    #[error("cannot read from stderr: {0}")]
    CannotReadFromStderr(std::io::Error),
    #[error("cannot get current directory: {0}")]
    CannotGetCwd(std::io::Error),

    #[error("task \"{0}\" not found")]
    TaskNotFound(String),
}
