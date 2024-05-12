//! Custom error types.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

use crate::vobsub::NomError;

/// A type representing errors that are specific to `subtile`. Note that we may
/// normally return `Error`, not `SubError`, which allows to return other
/// kinds of errors from third-party libraries.
#[derive(Debug, Error)]
pub enum SubError {
    /// We were unable to find a required key in an `*.idx` file.
    #[error("Could not find required key '{0}'")]
    MissingKey(&'static str),

    /// We could not parse a value.
    #[error("Could not parse: {0}")]
    Parse(String),

    /// We could not process a subtitle image.
    #[error("Could not process subtitle image: {0}")]
    Image(String),

    /// If an error happen during parsing with `nom`.
    #[error("Parsing error.")]
    NomParsing(#[from] NomError),

    /// We could not read a file.
    #[error("Could not read '{path}'")]
    Io {
        /// Source error
        source: io::Error,
        /// Path of the file we tried to read
        path: PathBuf,
    },
}
