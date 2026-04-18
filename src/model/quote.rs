/// Quote form used to represent a binding value in source text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuoteType {
    /// Unquoted value.
    None,
    /// Single-quoted value.
    Single,
    /// Double-quoted value with supported escape decoding.
    Double,
}
