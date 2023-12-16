//! This module reads DVD subtitles in VobSub format.  These are typically
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
//!     let coords = sub.coordinates();
//!     println!("At: {}, {}", coords.left(), coords.top());
//!     println!("Size: {}x{}", coords.width(), coords.height());
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
//! VobSub subtitles consist of a simple textual `*.idx` file, and a binary
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
//!   stored in an IFO file.
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

pub use crate::{Error, Result};

pub use self::idx::Index;
pub use self::palette::{palette, Palette};
pub use self::probe::{is_idx_file, is_sub_file};
pub use self::sub::{subtitles, Coordinates, Subtitle, Subtitles};
