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
//! use crate::subtile::{
//!     image::{ImageSize, ImageArea, ToImage},
//!     time::TimeSpan,
//!     vobsub::{conv_to_rgba, VobSubIndexedImage, VobSubToImage},
//! };
//!
//! let idx = subtile::vobsub::Index::open("./fixtures/example.idx").unwrap();
//! for sub in idx.subtitles::<(TimeSpan, VobSubIndexedImage)>() {
//!     let (time_span, image) = sub.unwrap();
//!     println!("Time: {:0.3}-{:0.3}", time_span.start, time_span.end);
//!     //println!("Always show: {:?}", sub.force());
//!     let area = image.area();
//!     println!("At: {}, {}", area.left(), area.top());
//!     println!("Size: {}x{}", image.width(), image.height());
//!     let img: image::RgbaImage = VobSubToImage::new(&image, idx.palette(), conv_to_rgba).to_image();
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
//! - [Packetized Elementary Stream]`PES` (`PES`)
//! - [DVD subtitles](http://sam.zoy.org/writings/dvd/subtitles/)
//! - [System Time Clock](http://www.bretl.com/mpeghtml/STC.HTM)
//!
//! [PES]: http://dvd.sourceforge.net/dvdinfo/pes-hdr.html
//!
//! There are also any number of open source implementations of subtitles
//! decoders which might be useful once you get past the Program Stream and
//! `PES` wrappers.
//!
//! There are two closely-related formats that this library could be
//! extended to parse without too much work:
//!
//! - Subtitles embedded in DVD-format video.  These should contain the
//!   same subtitle packet format, but the `*.idx` file is replaced by data
//!   stored in an `IFO` file.
//! - Subtitles stored in the `Matroska` container format.  Again, these use
//!   the same basic subtitle format, but the `*.idx` file is replaced by
//!   an internal, stripped-down version of the same data in text format.
//!

mod decoder;
mod idx;
mod img;
mod modifier;
mod mpeg2;
mod palette;
mod probe;
mod sub;

pub use self::{
    idx::{read_palette, Index},
    img::{conv_to_rgba, VobSubIndexedImage, VobSubOcrImage, VobSubToImage},
    modifier::VobsubModifier,
    palette::{palette, palette_rgb_to_luminance, Palette},
    probe::{is_idx_file, is_sub_file},
    sub::ErrorMissing,
};

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

    /// If invalid number of palette entries found.
    #[error("Palette must have 16 entries, found '{0}' one")]
    PaletteInvalidEntriesNumbers(usize),

    /// Parsing of palette in `*.idx` file failed.
    #[error("Error during palette parsing from .idx file")]
    PaletteError(#[source] NomError),

    /// If Scan line offsets values are not correct.
    #[error("invalid scan line offsets : start 0 {start_0}, start 1 {start_1}, end {end}")]
    InvalidScanLineOffsets {
        /// Start 0
        start_0: usize,
        /// Start 1
        start_1: usize,
        /// End
        end: usize,
    },

    /// If the buffer is too Small for parsing a 16-bits value.
    #[error("unexpected end of buffer while parsing 16-bit size")]
    BufferTooSmallForU16,

    /// If the buffer is too small to parse a subtitle.
    #[error("unexpected end of subtitle data")]
    UnexpectedEndOfSubtitleData,

    /// If an error happen during `Control sequence` parsing.
    #[error("Error with Control sequence parsing.")]
    ControlSequence(#[source] NomError),

    /// If the control offset value tried to leads backwards.
    #[error("control offset value tried to leads backwards")]
    ControlOffsetWentBackwards,

    /// If `control offset` is bigger than packet size.
    #[error("control offset is 0x{offset:x}, but packet is only 0x{packet:x} bytes")]
    ControlOffsetBiggerThanPacket {
        /// Control offset
        offset: usize,
        /// Packet size
        packet: usize,
    },

    /// If an error happen during `PES Packet` parsing.
    #[error("PES packet parsing.")]
    PESPacket(#[source] NomError),

    /// If the `control packet` is incomplete
    #[error("Incomplete control packet")]
    IncompleteControlPacket,

    /// Packet is too short, not bigger to read his size.
    #[error("Packet is too short")]
    PacketTooShort,

    /// If timing info for Subtitle is missing.
    #[error("found subtitle without timing into")]
    MissingTimingForSubtitle,

    /// Missing data from parsing to construct a subtitle.
    #[error("Missing during subtitle parsing")]
    MissingSubtitleParsing(#[from] ErrorMissing),

    /// We could not process a subtitle image.
    #[error("Could not process subtitle image: {0}")]
    Image(#[from] img::Error),

    /// Io error on a path.
    #[error("Io error on '{path}'")]
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

    /// An Error occurred during parsing
    #[error("Error from nom : {0}")]
    Error(String),

    /// An Failure occurred during parsing
    #[error("Failure from nom : {0}")]
    Failure(String),
}

/// Extend `IResult` management, and convert to [`Result`] with [`NomError`]
pub trait IResultExt<I, O, E> {
    /// Convert an `IResult` to Result<_, `NomError`> and check than the buffer is empty after parsing.
    /// # Errors
    /// Forward `Error` and `Failure` from `nom`, and return `UnexpectedInput` if the buffer is not empty after parsing.
    fn to_result_no_rest(self) -> Result<O, NomError>;

    /// Convert an `IResult` to Result<_, `NomError`>
    /// # Errors
    /// Forward `Error` and `Failure` from `nom`.
    fn to_result(self) -> Result<(I, O), NomError>;
}

impl<I: Default + Eq, O, E: fmt::Debug> IResultExt<I, O, E> for IResult<I, O, E> {
    fn to_result_no_rest(self) -> Result<O, NomError> {
        match self {
            Self::Ok((rest, val)) => {
                if rest == I::default() {
                    Ok(val)
                } else {
                    Err(NomError::UnexpectedInput)
                }
            }
            Self::Err(err) => match err {
                nom::Err::Incomplete(needed) => Err(NomError::IncompleteInput(needed)),
                nom::Err::Error(err) => Err(NomError::Error(format!("{err:?}"))),
                nom::Err::Failure(err) => Err(NomError::Failure(format!("{err:?}"))),
            },
        }
    }
    fn to_result(self) -> Result<(I, O), NomError> {
        match self {
            Self::Ok((rest, val)) => Ok((rest, val)),
            Self::Err(err) => match err {
                nom::Err::Incomplete(needed) => Err(NomError::IncompleteInput(needed)),
                nom::Err::Error(err) => Err(NomError::Error(format!("{err:?}"))),
                nom::Err::Failure(err) => Err(NomError::Failure(format!("{err:?}"))),
            },
        }
    }
}
