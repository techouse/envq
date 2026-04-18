use std::ffi::OsString;
use std::io::{self, Cursor, Write};

use super::run_process;

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn run_process_writes_raw_streams_and_returns_exit_code() {
    let mut stdin = Cursor::new(Vec::new());
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run_process(
        [OsString::from("--version")],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 0);
    assert!(stdout.starts_with(b"envq "));
    assert!(stderr.is_empty());
}

#[test]
fn run_process_reports_stream_write_failure() {
    let mut stdin = Cursor::new(Vec::new());
    let mut stdout = FailingWriter;
    let mut stderr = Vec::new();

    let code = run_process(
        [OsString::from("--version")],
        &mut stdin,
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(code, 1);
}
