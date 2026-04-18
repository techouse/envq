#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod cli;
pub mod diagnostics;
pub(crate) mod diff;
pub mod editor;
pub mod io_atomic;
pub mod model;
pub mod parser;
pub mod render;

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub mod fuzzing {
    use std::path::Path;

    /// Exposes internal diff generation to the out-of-crate fuzz workspace.
    #[must_use]
    pub fn unified_diff(path: &Path, before: &[u8], after: &[u8]) -> Vec<u8> {
        crate::diff::unified_diff(path, before, after)
    }
}

#[cfg(all(test, feature = "fuzzing"))]
mod tests {
    use std::path::Path;

    #[test]
    fn fuzzing_diff_wrapper_delegates_to_internal_diff() {
        assert_eq!(
            crate::fuzzing::unified_diff(Path::new("x.env"), b"A=1\n", b"A=2\n"),
            crate::diff::unified_diff(Path::new("x.env"), b"A=1\n", b"A=2\n")
        );
    }
}
