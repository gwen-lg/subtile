//! This module reads DVD subtitles in `VobSub` format.  These are typically
//! stored as two files: an `*.idx` file summarizing the subtitles, and an
//! MPEG-2 Program Stream containing the actual subtitle packets.
//!
//! ## Example code
//!
//! ```
//! extern crate image;
//! extern crate subtile;
//!
//! let idx = subtile::vobsub::Index::open("./fixtures/example.idx").unwrap();
//! for sub in idx.subtitles() {
//!     let sub = sub.unwrap();
//!     println!("Time: {:0.3}-{:0.3}", sub.start_time(), sub.end_time());
//!     println!("Always show: {:?}", sub.force());
//!     let area = sub.area();
//!     println!("At: {}, {}", area.left(), area.top());
//!     println!("Size: {}x{}", area.width(), area.height());
//!     let img: image::RgbaImage = sub.to_image(idx.palette());
//!
//!     // You can save or manipulate `img` using the APIs provided by the Rust
//!     // `image` crate.
//! }
//! ```
//! ## Limitations
//!
//! The initial version of this library is focused on extracting just the
//! information shown above, and it does not have full support for all the
//! options found in `*.idx` files.  It also lacks support for rapidly
//! finding the subtitle associated with a particular time during playback.
//!
//! ## Background & References
//!
//! `VobSub` subtitles consist of a simple textual `*.idx` file, and a binary
//! `*.sub` file.  The binary `*.sub` file is essentially an MPEG-2 Program
//! Stream containing Packetized Elementary Stream data, but only for a
//! single subtitle track.
//!
//! Useful references include:
//!
//! - [Program Stream](https://en.wikipedia.org/wiki/MPEG_program_stream) (PS)
//! - [Packetized Elementary Stream][PES] (PES)
//! - [DVD subtitles](http://sam.zoy.org/writings/dvd/subtitles/)
//! - [System Time Clock](http://www.bretl.com/mpeghtml/STC.HTM)
//!
//! [PES]: http://dvd.sourceforge.net/dvdinfo/pes-hdr.html
//!
//! There are also any number of open source implementations of subtitles
//! decoders which might be useful once you get past the Program Stream and
//! PES wrappers.
//!
//! There are two closely-related formats that this library could be
//! extended to parse without too much work:
//!
//! - Subtitles embedded in DVD-format video.  These should contain the
//!   same subtitle packet format, but the `*.idx` file is replaced by data
//!   stored in an `IFO` file.
//! - Subtitles stored in the Matroska container format.  Again, these use
//!   the same basic subtitle format, but the `*.idx` file is replaced by
//!   an internal, stripped-down version of the same data in text format.
//!

mod idx;
mod img;
mod mpeg2;
mod palette;
mod probe;
mod sub;

pub use self::idx::{read_palette, Index};
pub use self::palette::{palette, Palette};
pub use self::probe::{is_idx_file, is_sub_file};
pub use self::sub::{subtitles, Subtitle, Subtitles};

use crate::content::ContentError;
use nom::{IResult, Needed};
use std::{fmt, io, path::PathBuf};
use thiserror::Error;

/// Error for `VobSub` handling.
#[derive(Debug, Error)]
pub enum VobSubError {
    /// Content Error
    #[error("Error with data")]
    Content(#[from] ContentError),

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

/// Error from `nom` handling
#[derive(Debug, Error)]
pub enum NomError {
    /// We have leftover input that we didn't expect.
    #[error("Unexpected extra input")]
    UnexpectedInput,

    /// Our input data ended sooner than we expected.
    #[error("Incomplete input: '{0:?}' needed.")]
    IncompleteInput(Needed),

    /// An error happend during parsing
    #[error("Error from nom : {0}")]
    Error(String),

    /// And Failure happend during parsing
    #[error("Failure from nom : {0}")]
    Failure(String),
}

/// Extend `IResult` management, and convert to [`Result`] with [`NomError`]
pub trait IResultExt<I, O, E> {
    /// Convert an `IResult` to Result<_, `NomError`> and check than the buffer is empty after parsing.
    /// # Errors
    /// Forward `Error` and `Failure` from nom, and return `UnexpectedInput` if the buffer is not empty after parsing.
    fn to_result_no_rest(self) -> Result<O, NomError>;

    /// Convert an `IResult` to Result<_, `NomError`>
    /// # Errors
    /// Forward `Error` and `Failure` from nom.
    fn to_result(self) -> Result<(I, O), NomError>;
}

impl<I: Default + Eq, O, E: fmt::Debug> IResultExt<I, O, E> for IResult<I, O, E> {
    fn to_result_no_rest(self) -> Result<O, NomError> {
        match self {
            IResult::Ok((rest, val)) => {
                if rest == I::default() {
                    Ok(val)
                } else {
                    Err(NomError::UnexpectedInput)
                }
            }
            IResult::Err(err) => match err {
                nom::Err::Incomplete(needed) => Err(NomError::IncompleteInput(needed)),
                nom::Err::Error(err) => Err(NomError::Error(format!("{err:?}"))),
                nom::Err::Failure(err) => Err(NomError::Failure(format!("{err:?}"))),
            },
        }
    }
    fn to_result(self) -> Result<(I, O), NomError> {
        match self {
            IResult::Ok((rest, val)) => Ok((rest, val)),
            IResult::Err(err) => match err {
                nom::Err::Incomplete(needed) => Err(NomError::IncompleteInput(needed)),
                nom::Err::Error(err) => Err(NomError::Error(format!("{err:?}"))),
                nom::Err::Failure(err) => Err(NomError::Failure(format!("{err:?}"))),
            },
        }
    }
}
