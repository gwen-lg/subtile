use thiserror::Error;

use super::{PgsError, ReadExt as _};
use std::{
    array::TryFromSliceError,
    fmt,
    io::{BufRead, ErrorKind, Seek},
};

// Segment start Magic Number
const MAGIC_NUMBER: [u8; 2] = [0x50, 0x47];

/// Represent a valid `SegmentType`.
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

/// Wrap Bytes of a segment buffer (as read from matroska by example).
///
/// It's used by [`SegmentSplitter`] to return data.
pub struct SegmentBuf<'a> {
    buffer: &'a [u8],
}
impl<'a> SegmentBuf<'a> {
    /// Get code of the segment.
    ///
    /// # Panics
    ///
    /// Should never panic, as the `SegmentTypeCode` is checked at construction.
    #[must_use]
    pub fn code(&self) -> SegmentTypeCode {
        SegmentTypeCode::try_from(self.buffer[0]).unwrap()
    }
    /// Get the buffer of the segment. This include segment code and size as header.
    #[must_use]
    pub const fn buffer(&self) -> &'a [u8] {
        self.buffer
    }
    /// Get the data of the segment. Doesn't include segment code and size.
    #[must_use]
    pub fn data(&self) -> &'a [u8] {
        &self.buffer[3..]
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

/// Define the errors of Segment Buffer creation.
#[derive(Debug, Error)]
pub enum SegmentSplitterError {
    #[error(transparent)]
    TypeCode(#[from] PgsError),

    #[error("Invalid segment size found")]
    Size(#[source] TryFromSliceError),

    #[error("Segment Buffer creation Failed")]
    BufCreation(#[source] SegmentBufError),
}

/// This split a buffer of segment into [`SegmentBuf`].
///
/// It implement [`Iterator`] of Segment on a buffer.
/// This can be used with data `PGS` in matroska files.
#[derive(Debug, Clone, Copy)]
pub struct SegmentSplitter<'a> {
    content: &'a [u8],
}

impl<'a> SegmentSplitter<'a> {
    fn split_next(&mut self) -> Result<SegmentBuf<'a>, SegmentSplitterError> {
        let _seg_code =
            SegmentTypeCode::try_from(self.content[0]).map_err(SegmentSplitterError::TypeCode)?;
        let buf = self.content[1..3]
            .try_into()
            .map_err(SegmentSplitterError::Size)?;
        let seg_size = u16::from_be_bytes(buf);

        // + 3 to take the header size into account
        let (seg_data, remain) = self.content.split_at(seg_size as usize + 3);
        self.content = remain;

        SegmentBuf::try_from(seg_data).map_err(SegmentSplitterError::BufCreation)
    }
}

impl<'a> From<&'a [u8]> for SegmentSplitter<'a> {
    fn from(content: &'a [u8]) -> Self {
        Self { content }
    }
}

impl<'a> Iterator for SegmentSplitter<'a> {
    type Item = Result<SegmentBuf<'a>, SegmentSplitterError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.content.is_empty() {
            None
        } else {
            Some(self.split_next())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{self},
        path::Path,
    };

    #[test]
    fn segment_type_code_valid() {
        assert_eq!(
            SegmentTypeCode::try_from(0x14).unwrap(),
            SegmentTypeCode::Pds
        );
        assert_eq!(
            SegmentTypeCode::try_from(0x15).unwrap(),
            SegmentTypeCode::Ods
        );
        assert_eq!(
            SegmentTypeCode::try_from(0x16).unwrap(),
            SegmentTypeCode::Pcs
        );
        assert_eq!(
            SegmentTypeCode::try_from(0x17).unwrap(),
            SegmentTypeCode::Wds
        );
        assert_eq!(
            SegmentTypeCode::try_from(0x80).unwrap(),
            SegmentTypeCode::End
        );
    }

    #[test]
    fn segment_type_code_invalid() {
        assert!(matches!(
            SegmentTypeCode::try_from(0x00),
            Err(PgsError::SegmentInvalidTypeCode { value } ) if value == 0x00));
        assert!(matches!(
            SegmentTypeCode::try_from(0x13),
            Err(PgsError::SegmentInvalidTypeCode { value } ) if value == 0x13));
        assert!(matches!(
            SegmentTypeCode::try_from(0x18),
            Err(PgsError::SegmentInvalidTypeCode { value } ) if value == 0x18));
        assert!(matches!(
            SegmentTypeCode::try_from(0x79),
            Err(PgsError::SegmentInvalidTypeCode { value } ) if value == 0x79));
        assert!(matches!(
            SegmentTypeCode::try_from(0x81),
            Err(PgsError::SegmentInvalidTypeCode { value } ) if value == 0x81));
    }

    #[test]
    fn segment_type_code_str() {
        let pds_str: &'static str = SegmentTypeCode::Pds.into();
        assert_eq!(pds_str, "PDS");
        let ods_str: &'static str = SegmentTypeCode::Ods.into();
        assert_eq!(ods_str, "ODS");
        let pcs_str: &'static str = SegmentTypeCode::Pcs.into();
        assert_eq!(pcs_str, "PCS");
        let wds_str: &'static str = SegmentTypeCode::Wds.into();
        assert_eq!(wds_str, "WDS");
        let end_str: &'static str = SegmentTypeCode::End.into();
        assert_eq!(end_str, "END");
    }

    #[test]
    fn segment_buf_pds() {
        let buf: [u8; 7] = [0x14, 0x00, 0x04, 0x10, 0x25, 0x06, 0x00];
        let seg = SegmentBuf::try_from(buf.as_slice()).unwrap();
        assert_eq!(seg.code(), SegmentTypeCode::Pds);
        assert_eq!(seg.data().len(), 4);
        assert_eq!(seg.data(), buf[3..].iter().as_slice());
    }

    #[test]
    fn segment_buf_end() {
        let buf: [u8; 3] = [0x80, 0x00, 0x00];
        let seg = SegmentBuf::try_from(buf.as_slice()).unwrap();

        assert_eq!(seg.code(), SegmentTypeCode::End);
        assert!(seg.data().is_empty());
    }

    fn segment_splitter_test_sub_start(path: impl AsRef<Path>) {
        let buf = fs::read(path).unwrap();

        let seg_splitter = SegmentSplitter::from(buf.as_slice());
        seg_splitter.for_each(|seg_buf| {
            assert!(seg_buf.is_ok());
        });
        assert_eq!(seg_splitter.count(), 5);
    }
    fn segment_splitter_test_sub_end(path: impl AsRef<Path>) {
        let buf = fs::read(path).unwrap();

        let seg_splitter = SegmentSplitter::from(buf.as_slice());
        seg_splitter.for_each(|seg_buf| {
            assert!(seg_buf.is_ok());
        });
        assert_eq!(seg_splitter.count(), 3);
    }

    #[test]
    fn segment_splitter_580() {
        segment_splitter_test_sub_start("fixtures/pgs/segments_580.raw");
    }

    #[test]
    fn segment_splitter_2400() {
        segment_splitter_test_sub_end("fixtures/pgs/segments_2400.raw");
    }

    #[test]
    fn segment_splitter_2540() {
        segment_splitter_test_sub_start("fixtures/pgs/segments_2540.raw");
    }
    #[test]
    fn segment_splitter_4760() {
        segment_splitter_test_sub_end("fixtures/pgs/segments_4760.raw");
    }
}
