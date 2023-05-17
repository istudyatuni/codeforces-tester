use std::{
    env::current_dir,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::{Error, Result};

pub(crate) fn exec<S: Into<String>>(cmd: S, cwd: &Option<PathBuf>) -> Result<()> {
    let conf = prepare_exec(cmd, cwd)?;
    let status = Command::new(conf.name)
        .args(conf.args)
        .current_dir(conf.cwd)
        .stderr(Stdio::null())
        .status()
        .map_err(Error::CannotCreateCommand)?;
    if status.success() {
        return Ok(());
    }

    match status.code() {
        Some(c) => Err(Error::CommandExited(c)),
        None => Err(Error::CommandTerminated),
    }
}

pub(crate) fn exec_with_io<S: Into<String>>(
    cmd: S,
    input: S,
    cwd: &Option<PathBuf>,
) -> Result<String> {
    let conf = prepare_exec(cmd, cwd)?;
    let child = Command::new(conf.name)
        .args(conf.args)
        .current_dir(conf.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(Error::CannotCreateCommand)?;
    let mut stdin = child.stdin.expect("cannot get stdin");
    let mut stdout = child.stdout.expect("cannot get stdout");
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

    Ok(output)
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

#[derive(Debug)]
struct CommandConfig {
    name: String,
    args: Vec<String>,
    cwd: PathBuf,
}
