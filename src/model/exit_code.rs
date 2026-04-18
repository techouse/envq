/// Process exit codes used by the CLI compatibility contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitCode {
    /// Command completed successfully.
    Success = 0,
    /// Usage, I/O, or other general failure.
    GeneralError = 1,
    /// Requested key was absent.
    KeyNotFound = 2,
    /// Input failed validation before file I/O.
    ValidationError = 3,
    /// `--check` found that a mutation would change output.
    WouldChange = 4,
}

impl ExitCode {
    /// Numeric process status code.
    #[must_use]
    pub const fn code(self) -> i32 {
        self as i32
    }
}
