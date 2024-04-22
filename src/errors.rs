//! Custom error types.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// A type representing errors that are specific to `subtile`. Note that we may
/// normally return `Error`, not `SubError`, which allows to return other
/// kinds of errors from third-party libraries.
#[derive(Debug, Error)]
pub enum SubError {
    /// Our input data ended sooner than we expected.
    #[error("Input ended unexpectedly")]
    IncompleteInput,

    /// We were unable to find a required key in an `*.idx` file.
    #[error("Could not find required key '{0}'")]
    MissingKey(&'static str),

    /// We could not parse a value.
    #[error("Could not parse: {0}")]
    Parse(String),

    /// We could not process a subtitle image.
    #[error("Could not process subtitle image: {0}")]
    Image(String),

    /// We have leftover input that we didn't expect.
    #[error("Unexpected extra input")]
    UnexpectedInput,

    /// We could not read a file.
    #[error("Could not read '{path}'")]
    Io {
        /// Source error
        source: io::Error,
        /// Path of the file we tried to read
        path: PathBuf,
    },
}
