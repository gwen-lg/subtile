//! Custom error types.

use nom::IResult;
use std::default::Default;
use std::fmt;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// A type representing errors that are specific to `subtitles-utils`. Note that we may
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

pub trait IResultExt<I, O, E> {
    fn ignore_trailing_data(self) -> IResult<I, O, E>;
    fn to_vobsub_result(self) -> Result<O, SubError>;
}

impl<I: Default + Eq, O, E: fmt::Debug> IResultExt<I, O, E> for IResult<I, O, E> {
    fn ignore_trailing_data(self) -> IResult<I, O, E> {
        match self {
            IResult::Ok((_, val)) => IResult::Ok((I::default(), val)),
            other => other,
        }
    }

    fn to_vobsub_result(self) -> Result<O, SubError> {
        match self {
            IResult::Ok((rest, val)) => {
                if rest == I::default() {
                    Ok(val)
                } else {
                    Err(SubError::UnexpectedInput)
                }
            }
            IResult::Err(err) => match err {
                nom::Err::Incomplete(_) => Err(SubError::IncompleteInput),
                nom::Err::Error(err) => Err(SubError::Parse(format!("{:?}", err))),
                nom::Err::Failure(err) => Err(SubError::Parse(format!("{:?}", err))),
            },
        }
    }
}
