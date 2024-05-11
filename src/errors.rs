//! Custom error types.

use thiserror::Error;

/// A type representing errors that are specific to `subtile`. Note that we may
/// normally return `Error`, not `SubError`, which allows to return other
/// kinds of errors from third-party libraries.
#[derive(Debug, Error)]
pub enum SubError {
    /// Error with `VobSub`
    #[error("Error with VobSub")]
    VobSub(#[from] crate::vobsub::VobSubError),
}
