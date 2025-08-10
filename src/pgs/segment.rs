use super::{PgsError, ReadExt as _};
use std::{
    fmt,
    io::{BufRead, ErrorKind, Seek},
};

// Segment start Magic Number
const MAGIC_NUMBER: [u8; 2] = [0x50, 0x47];

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SegmentTypeCode {
    Pds = 0x14,
    Ods = 0x15,
    Pcs = 0x16,
    Wds = 0x17,
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
