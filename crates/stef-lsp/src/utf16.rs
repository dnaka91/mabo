//! Utilities for handling UTF-16 encoding of strings.
//!
//! As the LSP by default uses UTF-16 and Rust strings are encoded as UTF-8, special handling is
//! required to calculate the correct information. Mostly these are adjustments to character
//! offsets.

/// Get the UTF-16 byte count for the code line.
pub fn len(line: &str) -> usize {
    line.encode_utf16().count()
}
