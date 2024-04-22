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

use crate::SubError;
use core::fmt;
use nom::IResult;

/// Extend `IResult` management, and convert to [`Result`] with [`Subrror`]
pub trait IResultExt<I, O, E> {
    /// Forward IResult after trailing remaining data.
    /// # Errors
    /// Forward `Error` and `Failure` from nom.
    fn ignore_trailing_data(self) -> IResult<I, O, E>;
    /// Convert an `IResult` to Result<_, `SubError`>
    /// # Errors
    /// return `UnexpectedInput` if there is trailing data after parsing.
    /// Forward `Error` and `Failure` from nom.
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
                nom::Err::Error(err) | nom::Err::Failure(err) => {
                    Err(SubError::Parse(format!("{err:?}")))
                }
            },
        }
    }
}
