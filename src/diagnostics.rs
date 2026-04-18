//! Validation and diagnostic helpers shared by parser and CLI layers.

/// Validates envq's supported key grammar.
#[must_use]
pub fn is_valid_key(key: &str) -> bool {
    let mut chars = key.bytes();
    let Some(first) = chars.next() else {
        return false;
    };
    if !is_key_start(first) {
        return false;
    }
    chars.all(is_key_continue)
}

/// Formats the contract-compatible invalid-key diagnostic.
#[must_use]
pub fn invalid_key_message(key: &str) -> Vec<u8> {
    format!("envq: invalid key: {key}\n").into_bytes()
}

/// Returns whether a byte can start a supported key.
#[must_use]
pub(crate) const fn is_key_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

/// Returns whether a byte can continue a supported key.
#[must_use]
pub(crate) const fn is_key_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests;
