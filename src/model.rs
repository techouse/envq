//! Byte-backed document model shared by the parser, editor, and renderer.

mod document;
mod exit_code;
mod line;
mod quote;

pub use document::Document;
pub use exit_code::ExitCode;
pub use line::{BindingLine, BlankLine, CommentLine, InvalidLine, Line};
pub use quote::QuoteType;

#[cfg(test)]
mod tests;
