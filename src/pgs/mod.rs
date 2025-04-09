//! Read functionalities for Presentation Graphic Stream (.sup)
//!
//! Presentation Graphic Stream (SUP files) `BluRay` Subtitle Format doc :
//! <https://blog.thescorpius.com/index.php/2017/07/15/presentation-graphic-stream-sup-files-bluray-subtitle-format/>
//!
mod decoder;
mod ods;
mod pds;
mod pgs_image;
mod segment;
mod sup;
mod u24;

pub use decoder::{DecodeTimeImage, DecodeTimeOnly, PgsDecoder};
use ods::ObjectDefinitionSegment;
pub use pgs_image::{pixel_pass_through, RleEncodedImage, RleToImage};
pub use sup::SupParser;

pub use self::segment::SegmentTypeCode;
use std::{
    io::{self, BufRead, Cursor, Seek},
    path::PathBuf,
};
use thiserror::Error;

/// Error for `Pgs` handling.
#[derive(Debug, Error)]
pub enum PgsError {
    /// Io error on a path.
    #[error("Io error on '{path}'")]
    Io {
        /// Source error
        source: io::Error,
        /// Path of the file we tried to read
        path: PathBuf,
    },

    /// Encapsulates errors from `Object Definition Segment` parsing.
    #[error("Object Definition Segment parsing")]
    ODSParse(#[from] ods::Error),

    /// Encapsulates errors from `Palette Definition Segment` parsing.
    #[error("Palette Definition Segment parsing")]
    PDSParse(#[from] pds::Error),

    /// Invalid segment type code value.
    #[error("Invalid value '{value:#02x}' for Segment Type Code ")]
    SegmentInvalidTypeCode {
        /// Value tried to be Interpréted in Segment Type.
        value: u8,
    },

    /// An error occurred during Segment Header reading.
    #[error("Failed to read a complete segment header.")]
    SegmentFailReadHeader,

    /// Missing expected `PG` Magic number.
    #[error("Unable to read segment - PG missing!")]
    SegmentPGMissing,

    /// `ReadError` occurred during skipping the segment.
    #[error("Skipping Segment {type_code}")]
    SegmentSkip {
        /// Parent `ReadError`
        #[source]
        source: ReadError,
        /// type code of the segment we skip
        type_code: SegmentTypeCode,
    },

    /// Error if image is missing to complete the parsing of a subtitle.
    #[error("Missing image during `Presentation Graphic Stream (PGS)` parsing")]
    MissingImage,

    /// Palette is missing after image parsing.
    #[error("Missing palette after image parsing")]
    MissingPalette,
}

/// Error from data read for parsing.
#[derive(Debug, Error)]
pub enum ReadError {
    /// Reading of the buffer have failed.
    #[error("Failed read buffer of size : {buffer_size}")]
    FailedReadBuffer {
        /// `io` error
        #[source]
        source: io::Error,
        /// size of the buffer
        buffer_size: usize,
    },

    /// An error has occurred during buffer filling from reader.
    #[error("Failed to fill buffer from Reader")]
    FailedFillBuf(#[source] io::Error),

    /// An error has occurred during seek in reader.
    #[error("Seek failed")]
    FailedSeek(#[source] io::Error),
}

/// Super-trait of `BufRead` + `Seek` to extend reading functionalities useful for parsing.
pub trait ReadExt
where
    Self: BufRead + Seek,
{
    /// Read a buffer from a reader with error management.
    ///
    /// # Errors
    ///
    /// Will return `FailedReadBuffer` if `read_exact` failed.
    fn read_buffer(&mut self, to_read: &mut [u8]) -> Result<(), ReadError> {
        self.read_exact(to_read)
            .map_err(|source| ReadError::FailedReadBuffer {
                source,
                buffer_size: to_read.len(),
            })
    }

    /// Skip data from a reader with error management.
    ///
    /// # Errors
    ///
    /// Will return `FailedFillBuf` if `fill_buf` failed.
    /// `FailedSeek` if `seek` failed.
    fn skip_data(&mut self, to_skip: usize) -> Result<(), ReadError> {
        let buff = self.fill_buf().map_err(ReadError::FailedFillBuf)?;

        if buff.len() >= to_skip {
            self.consume(to_skip);
        } else {
            self.seek_relative(to_skip as i64)
                .map_err(ReadError::FailedSeek)?;
        }
        Ok(())
    }
}
impl<U> ReadExt for U where U: BufRead + Seek {}

/// TODO
#[derive(Debug, Default)]
pub struct SegmentSplitter<'a> {
    pds_data: Option<&'a [u8]>,
    ods_data: Option<&'a [u8]>,
    complete: bool,
}

impl<'a> SegmentSplitter<'a> {
    /// TODO
    ///
    /// # Panics
    ///
    /// Panics if .
    pub fn process_segment(&mut self, code: SegmentTypeCode, segment_data: &'a [u8]) {
        match code {
            SegmentTypeCode::Pds => {
                assert!(self.pds_data.is_none());
                self.pds_data = Some(segment_data);
            }
            SegmentTypeCode::Ods => {
                assert!(self.ods_data.is_none());
                self.ods_data = Some(segment_data);
            }
            SegmentTypeCode::Pcs => todo!(),
            SegmentTypeCode::Wds => todo!(),
            SegmentTypeCode::End => self.complete = true,
        }
    }

    /// .
    ///
    /// # Panics
    ///
    /// TODO: replace panic with Error
    #[must_use]
    pub fn into_image(self) -> RleEncodedImage {
        let pds_size = self.pds_data.unwrap().len();
        let mut pds_data = Cursor::new(self.pds_data.unwrap());
        let pds = pds::read(&mut pds_data, pds_size).unwrap();

        let ods_data = self.ods_data.unwrap();
        if let ObjectDefinitionSegment::Complete(ods) =
            ods::read(&mut Cursor::new(ods_data), ods_data.len(), None).unwrap()
        {
            RleEncodedImage::new(ods.width, ods.height, pds.palette, ods.object_data)
        } else {
            panic!("the ObjectDefinitionSegment is attenden to be complete");
        }
    }
}
