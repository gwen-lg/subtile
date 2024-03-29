//! Try to guess the types of files on disk.

use std::fs;
use std::io::Read;
use std::path::Path;

use crate::SubError;

/// Internal helper function which looks for "magic" bytes at the start of
/// a file.
fn has_magic(path: &Path, magic: &[u8]) -> Result<bool, SubError> {
    let mkerr = |source| SubError::Io {
        source,
        path: path.into(),
    };

    let mut f = fs::File::open(path).map_err(mkerr)?;
    let mut bytes = vec![0; magic.len()];
    f.read_exact(&mut bytes).map_err(mkerr)?;
    Ok(magic == &bytes[..])
}

/// Does the specified path appear to point to an `*.idx` file?
pub fn is_idx_file<P: AsRef<Path>>(path: P) -> Result<bool, SubError> {
    has_magic(path.as_ref(), b"# VobSub index file")
}

/// Does the specified path appear to point to a `*.sub` file?
///
/// Note that this may (or may not) return false positives for certain
/// MPEG-2 related formats.
pub fn is_sub_file<P: AsRef<Path>>(path: P) -> Result<bool, SubError> {
    has_magic(path.as_ref(), &[0x00, 0x00, 0x01, 0xba])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_idx_files() {
        assert!(is_idx_file("./fixtures/tiny.idx").unwrap());
        assert!(!is_idx_file("./fixtures/tiny.sub").unwrap());
    }

    #[test]
    fn probe_sub_files() {
        assert!(is_sub_file("./fixtures/tiny.sub").unwrap());
        assert!(!is_sub_file("./fixtures/tiny.idx").unwrap());
    }
}
