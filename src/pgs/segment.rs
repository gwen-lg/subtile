use thiserror::Error;

use super::{PgsError, ReadExt as _};
use std::{
    array::TryFromSliceError,
    fmt,
    io::{BufRead, ErrorKind, Seek},
};

// Segment start Magic Number
const MAGIC_NUMBER: [u8; 2] = [0x50, 0x47];

/// The Segment Type TODO: improve doc
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SegmentTypeCode {
    /// Palette Definition Segment
    ///
    ///Used to define a palette for color conversion.
    Pds = 0x14,
    ///Object Definition Segment
    //
    ///This segment defines the graphics object (image).
    Ods = 0x15,
    /// Presentation Composition Segment
    ///
    /// Used for composing a sub picture.
    /// TODO: be able to parse it
    Pcs = 0x16,
    /// Window Definition Segment
    ///
    /// Used to define the rectangular area on the screen where the sub picture will be shown.
    Wds = 0x17,
    /// End Segment
    ///
    /// The end segment always has a segment size of zero and indicates the end of a Display Set (DS) definition.
    End = 0x80,
}

impl TryFrom<u8> for SegmentTypeCode {
    type Error = PgsError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x14 => Ok(Self::Pds),
            0x15 => Ok(Self::Ods),
            0x16 => Ok(Self::Pcs),
            0x17 => Ok(Self::Wds),
            0x80 => Ok(Self::End),
            _ => Err(PgsError::SegmentInvalidTypeCode { value }),
        }
    }
}
impl From<SegmentTypeCode> for u8 {
    fn from(val: SegmentTypeCode) -> Self {
        val as Self
    }
}
impl From<SegmentTypeCode> for &'static str {
    fn from(val: SegmentTypeCode) -> Self {
        match val {
            SegmentTypeCode::Pds => "PDS",
            SegmentTypeCode::Ods => "ODS",
            SegmentTypeCode::Pcs => "PCS",
            SegmentTypeCode::Wds => "WDS",
            SegmentTypeCode::End => "END",
        }
    }
}
impl fmt::Debug for SegmentTypeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex: u8 = (*self).into();
        write!(f, "{hex:#02x}-{self}")
    }
}
impl fmt::Display for SegmentTypeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let friendly: &str = (*self).into();
        write!(f, "{friendly}")
    }
}

/// Struct of segment header.
#[derive(Debug)]
pub(crate) struct SegmentHeader {
    /// Presentation Timestamp.
    pts: u32,
    /// Code of the Segment Type
    type_code: SegmentTypeCode,
    /// Size of the segment.
    size: u16,
}

impl SegmentHeader {
    pub const fn presentation_time(&self) -> u32 {
        self.pts / 90 // Return time in milliseconds
    }
    pub const fn type_code(&self) -> SegmentTypeCode {
        self.type_code
    }
    pub const fn size(&self) -> u16 {
        self.size
    }
}

/// Length of the segment Header
const HEADER_LEN: usize = 2 + 4 + 4 + 1 + 2;

/// Read the segment header
pub fn read_header<R: BufRead>(reader: &mut R) -> Result<Option<SegmentHeader>, PgsError> {
    let mut buffer = [0u8; HEADER_LEN];

    match reader.read_exact(&mut buffer) {
        Ok(()) => parse_segment_header(buffer),
        Err(err) if err.kind() == ErrorKind::UnexpectedEof => {
            // Buffer is empty, just return to end parsing
            Ok(None)
        }
        Err(err) => {
            println!("{err:?}");
            Err(PgsError::SegmentFailReadHeader)
        }
    }
}

fn parse_segment_header(buffer: [u8; HEADER_LEN]) -> Result<Option<SegmentHeader>, PgsError> {
    if buffer[0..2] != MAGIC_NUMBER {
        return Err(PgsError::SegmentPGMissing);
    }
    let pts = u32::from_be_bytes(buffer[2..6].try_into().unwrap());
    let type_code = SegmentTypeCode::try_from(buffer[10])?;
    let size = u16::from_be_bytes(buffer[11..13].try_into().unwrap());

    Ok(Some(SegmentHeader {
        pts,
        type_code,
        size,
    }))
}

/// skip segment
pub fn skip_segment<R: BufRead + Seek>(
    reader: &mut R,
    header: &SegmentHeader,
) -> Result<(), PgsError> {
    let data_size: usize = header.size() as usize;
    reader
        .skip_data(data_size)
        .map_err(|source| PgsError::SegmentSkip {
            source,
            type_code: header.type_code(),
        })
}

/// Define the errors of Segment Buffer creation.
#[derive(Debug, Error)]
pub enum SegmentBufError {
    #[error("Failed to read valid `SegmentCode` from buffer")]
    SegmentCodeRead(#[from] PgsError),

    #[error("Failed to read valid `segment size` from buffer")]
    SegmentSizeRead(#[from] TryFromSliceError),

    #[error("Buffer len ({buf_len}) and segment size({seg_size}) doesn't match")]
    BufferLen { seg_size: u16, buf_len: usize },
}

/// struct to wrap a segment buffer (as read from matroska by example)
/// TODO: and used in [`SegmentSplitter`]
pub struct SegmentBuf<'a> {
    buffer: &'a [u8],
}
impl<'a> SegmentBuf<'a> {
    /// Get code of the segment.
    pub fn code(&self) -> SegmentTypeCode {
        SegmentTypeCode::try_from(self.buffer[0]).unwrap()
    }
    /// Get the buffer of the segment. This include segment code and size as header.
    pub fn buffer(&self) -> &'a [u8] {
        self.buffer
    }
    /// Get the data of the segment. Doesn't include segment code and size.
    pub fn data(&self) -> &'a [u8] {
        &self.buffer[4..]
    }
}

impl<'a> TryFrom<&'a [u8]> for SegmentBuf<'a> {
    type Error = SegmentBufError;

    fn try_from(buffer: &'a [u8]) -> Result<Self, Self::Error> {
        let _seg_code =
            SegmentTypeCode::try_from(buffer[0]).map_err(SegmentBufError::SegmentCodeRead)?;
        let seg_size = u16::from_be_bytes(
            buffer[1..3]
                .try_into()
                .map_err(SegmentBufError::SegmentSizeRead)?,
        );
        if seg_size as usize + 3 < buffer.len() {
            Err(SegmentBufError::BufferLen {
                seg_size,
                buf_len: buffer.len(),
            })
        } else {
            Ok(Self { buffer })
        }
    }
}
