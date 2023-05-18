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
pub(crate) struct CommandOutput {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl CommandOutput {
    fn new(stdout: String, stderr: String) -> Self {
        Self { stdout, stderr }
    }
}

pub(crate) fn exec<S>(cmd: S, cwd: &Option<PathBuf>) -> Result<()>
where
    S: Into<String> + Clone,
{
    let conf = prepare_exec(cmd.clone(), cwd)?;
    let status = Command::new(conf.name)
        .args(conf.args)
        .current_dir(conf.cwd)
        .status()
        .map_err(|e| Error::CannotCreateCommand(cmd.into(), e))?;
    if status.success() {
        return Ok(());
    }

    match status.code() {
        Some(c) => Err(Error::CommandExited(c)),
        None => Err(Error::CommandTerminated),
    }
}

pub(crate) fn exec_with_io<S>(cmd: S, input: S, cwd: &Option<PathBuf>) -> Result<CommandOutput>
where
    S: Into<String> + Clone,
{
    let conf = prepare_exec(cmd.clone(), cwd)?;
    let child = Command::new(conf.name)
        .args(conf.args)
        .current_dir(conf.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::CannotCreateCommand(cmd.into(), e))?;
    let mut stdin = child.stdin.expect("cannot get stdin");
    let mut stdout = child.stdout.expect("cannot get stdout");
    let mut stderr = child.stderr.expect("cannot get stderr");
    let input: String = input.into();

    stdin
        .write(input.as_bytes())
        .map_err(Error::CannotWriteToStdin)?;
    // close stdin
    drop(stdin);
    let mut output = String::new();
    stdout
        .read_to_string(&mut output)
        .map_err(Error::CannotReadFromStdout)?;
    let mut err_output = String::new();
    stderr
        .read_to_string(&mut err_output)
        .map_err(Error::CannotReadFromStderr)?;

    Ok(CommandOutput::new(output, err_output))
}

fn prepare_exec<S: Into<String>>(cmd: S, cwd: &Option<PathBuf>) -> Result<CommandConfig> {
    let cmd: String = cmd.into();
    let cmd = cmd.split_whitespace();
    let name = match cmd.clone().next() {
        Some(c) => c,
        None => return Err(Error::EmptyCommand),
    };

    let curdir = current_dir().map_err(Error::CannotGetCwd)?;
    let cwd = match cwd.clone() {
        Some(d) => {
            if d.is_absolute() {
                d
            } else {
                curdir.join(d)
            }
        }
        None => curdir,
    };
    Ok(CommandConfig {
        name: name.into(),
        args: cmd.skip(1).map(Into::into).collect(),
        cwd,
    })
}
