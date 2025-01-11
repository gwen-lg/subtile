use super::{u24::u24, ReadError, ReadExt};
use std::{
    fmt::{Debug, Display},
    io::{self, BufRead, Seek},
};
use thiserror::Error;

/// Error `ODS` (Object Definition Segment) handling.
#[derive(Debug, Error)]
pub enum Error {
    /// Error while tried reading `LastInSequence` flag.
    #[error("Reading `LastInSequenceFlag` failed")]
    LastInSequenceFlagReadData(#[source] io::Error),

    /// Value read for `LastInSequence` flag is invalid.
    #[error("LastInSequenceFlag : '{value:02x}' is not a valid value")]
    LastInSequenceFlagInvalidValue { value: u8 },

    /// Value of flag `LastInSequence` is not managed by the current code.
    #[error("LastInSequenceFlag::'{0}' flag is not mananged.")]
    LastInSequenceFlagNotManaged(LastInSequenceFlag),

    /// Failed during `Object ID` and `Object Version Number` skipping.
    #[error("Skipping `Object ID` and `Object Version Number`")]
    SkipObjectIdAndVerNum(#[source] ReadError),

    /// Failed during read `Width` of the image.
    #[error("Read With of the image incarried by the `Object Definition Segment`(s)")]
    ReadWidth(#[source] io::Error),

    /// Failed during read `Height` of the image.
    #[error("Read Height of the image incarried by the `Object Definition Segment`(s)")]
    ReadHeight(#[source] io::Error),

    /// The read of object data failed.
    #[error("Try reading object data (buffer size: {buff_size})")]
    ObjectData {
        #[source]
        source: io::Error,
        buff_size: usize,
    },
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LastInSequenceFlag {
    Last = 0x40,
    First = 0x80,
    FirstAndLast = 0xC0,
}
impl TryFrom<u8> for LastInSequenceFlag {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x40 => Ok(Self::Last),
            0x80 => Ok(Self::First),
            0xC0 => Ok(Self::FirstAndLast),
            value => Err(Error::LastInSequenceFlagInvalidValue { value }),
        }
    }
}
impl From<LastInSequenceFlag> for u8 {
    fn from(val: LastInSequenceFlag) -> Self {
        val as Self
    }
}
impl From<LastInSequenceFlag> for &'static str {
    fn from(val: LastInSequenceFlag) -> Self {
        match val {
            LastInSequenceFlag::Last => "Last",
            LastInSequenceFlag::First => "First",
            LastInSequenceFlag::FirstAndLast => "First and last",
        }
    }
}
impl Debug for LastInSequenceFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex: u8 = (*self).into();
        write!(f, "{hex:#02x}-{self}")
    }
}
impl Display for LastInSequenceFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let friendly: &str = (*self).into();
        write!(f, "{friendly} in sequence")
    }
}

impl LastInSequenceFlag {
    fn read<Reader: BufRead + Seek>(reader: &mut Reader) -> Result<Self, Error> {
        let mut last_in_sequence_byte = [0];
        reader
            .read_exact(&mut last_in_sequence_byte)
            .map_err(Error::LastInSequenceFlagReadData)?;

        Self::try_from(last_in_sequence_byte[0])
    }
}

/// This segment defines the graphics object : it contain the image.
/// The `object_data` contain theimage data compressed using Run-length Encoding (RLE)
#[derive(Debug)]
pub struct ObjectDefinitionSegment {
    pub width: u16,
    pub height: u16,
    pub object_data: Vec<u8>,
}

pub fn read<Reader: BufRead + Seek>(
    reader: &mut Reader,
    segments_size: usize,
) -> Result<ObjectDefinitionSegment, Error> {
    handle_object_fields(reader)?;

    let last_in_sequence_flag = LastInSequenceFlag::read(reader)?;

    let mut buffer = [0; 3];
    reader.read_exact(&mut buffer).unwrap();
    let object_data_length = u24::from(<&[u8] as TryInto<[u8; 3]>>::try_into(&buffer).unwrap());
    let (width, height) = read_img_size(reader)?;

    if last_in_sequence_flag == LastInSequenceFlag::FirstAndLast {
        let data_size: usize = object_data_length.to_u32().try_into().unwrap();
        let data_size = data_size - 4; // don't know why for now !!! Object Data Length include Width + Height ?
        assert!(segments_size == 11 + data_size);

        let mut object_data = vec![0; data_size];
        reader
            .read_exact(object_data.as_mut_slice())
            .map_err(|source| Error::ObjectData {
                source,
                buff_size: object_data.len(),
            })?;

        Ok(ObjectDefinitionSegment {
            width,
            height,
            object_data,
        })
    } else {
        Err(Error::LastInSequenceFlagNotManaged(last_in_sequence_flag))
    }
}

// Handle `Object ID` and `Object Version Number` fields by skip it.
// They are not useful for current subtitle management.
fn handle_object_fields<Reader: BufRead + Seek>(reader: &mut Reader) -> Result<(), Error> {
    reader
        .skip_data(2 + 1)
        .map_err(Error::SkipObjectIdAndVerNum)?;
    Ok(())
}

// Read the image size (width and height) fields.
fn read_img_size<Reader: BufRead + Seek>(reader: &mut Reader) -> Result<(u16, u16), Error> {
    let mut buffer = [0; 2];
    reader.read_exact(&mut buffer).map_err(Error::ReadWidth)?;
    let width = u16::from_be_bytes(buffer);
    reader.read_exact(&mut buffer).map_err(Error::ReadHeight)?;
    let height = u16::from_be_bytes(buffer);
    Ok((width, height))
}
