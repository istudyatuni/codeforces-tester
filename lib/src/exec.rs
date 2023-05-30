use std::{
    env::current_dir,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{Error, Result};

#[derive(Debug)]
struct CommandConfig {
    name: String,
    args: Vec<String>,
    cwd: PathBuf,
}

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

impl CommandOutput {
    fn new(stdout: String, stderr: String, success: bool) -> Self {
        Self {
            stdout,
            stderr,
            success,
        }
    }
}

pub(crate) fn exec<S>(cmd: S, input: Option<S>, cwd: Option<PathBuf>) -> Result<CommandOutput>
where
    S: Into<String>,
{
    let cmd: String = cmd.into();
    let conf = prepare_exec(&cmd, cwd)?;
    let stdin = if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };
    let mut child = Command::new(conf.name)
        .args(conf.args)
        .current_dir(conf.cwd)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::CannotCreateCommand(cmd, e))?;
    let mut stdout = child.stdout.take().expect("cannot get stdout");
    let mut stderr = child.stderr.take().expect("cannot get stderr");

    if let Some(input) = input {
        let mut stdin = child.stdin.take().expect("cannot get stdin");
        let input: String = input.into();

        stdin
            .write(input.as_bytes())
            .map_err(Error::CannotWriteToStdin)?;
        // close stdin
        drop(stdin);
    }

    let mut output = String::new();
    stdout
        .read_to_string(&mut output)
        .map_err(Error::CannotReadFromStdout)?;
    let mut err_output = String::new();
    stderr
        .read_to_string(&mut err_output)
        .map_err(Error::CannotReadFromStderr)?;

    let is_success = child.wait().map(|s| s.success()).unwrap_or(false);
    Ok(CommandOutput::new(output, err_output, is_success))
}

fn prepare_exec<S: Into<String>>(cmd: S, cwd: Option<PathBuf>) -> Result<CommandConfig> {
    let cmd: String = cmd.into();
    let cmd = cmd.split_whitespace();
    let name = match cmd.clone().next() {
        Some(c) => c,
        None => return Err(Error::EmptyCommand),
    };

    let cwd = match cwd {
        Some(d) => d,
        None => current_dir().map_err(Error::CannotGetCwd)?,
    };
    Ok(CommandConfig {
        name: name.into(),
        args: cmd.skip(1).map(Into::into).collect(),
        cwd,
    })
}
