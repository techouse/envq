#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{self, Read, Write};

fn main() {
    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let exit_code = run_process(
        std::env::args_os().skip(1),
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );
    std::process::exit(exit_code);
}

fn run_process<I, S>(
    argv: I,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let result = envq::cli::run(argv, stdin);

    // Write raw bytes instead of using `print!` so invalid UTF-8 output survives.
    let stdout_result = stdout.write_all(&result.stdout);
    let stderr_result = stderr.write_all(&result.stderr);

    if stdout_result.is_err() || stderr_result.is_err() {
        return 1;
    }
    result.exit_code.code()
}

#[cfg(test)]
#[path = "main/tests.rs"]
mod tests;
