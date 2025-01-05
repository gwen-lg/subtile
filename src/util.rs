//! Miscellaneous utilities.

use std::fmt;

/// Wrapper to force a `&[u8]` to display as nicely-formatted hexadecimal
/// bytes with only the the first line or so of bytes shown.
pub struct BytesFormatter<'a>(pub &'a [u8]);

impl fmt::Debug for BytesFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let BytesFormatter(bytes) = *self;
        for byte in bytes.iter().take(16) {
            write!(f, "{byte:02x} ")?;
        }
        write!(f, "({} bytes)", bytes.len())?;
        Ok(())
    }
}
