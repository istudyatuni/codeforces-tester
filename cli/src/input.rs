use std::io::{stdin, stdout, Read, Write};

use anyhow::Result;

pub(crate) fn read_line_with_prompt(prompt: &str) -> Result<String> {
    print!("{prompt}");
    stdout().flush()?;

    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    Ok(buf)
}

pub(crate) fn read_until_eof_with_prompt(prompt: &str) -> Result<String> {
    print!("{prompt}");
    stdout().flush()?;

    let mut buf = String::new();
    stdin().read_to_string(&mut buf)?;
    Ok(buf)
}
