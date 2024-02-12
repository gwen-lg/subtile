use crate::time::{TimePoint, TimeSpan};
use std::io::{BufRead, Seek};

use super::{
    segment::{read_header, SegmentTypeCode},
    PgsError,
};

/// Trait of `Presentation Graphic Stream` decoding.
pub trait PgsDecoder {
    /// Type of the Output data for the image.
    type Output;

    /// Parse next subtitle `PGS` and return an `Output` value.
    /// The `Output` depending of the data we want to decode.
    ///
    /// # Errors
    /// Return the error happened during parsing or decoding.
    fn parse_next<R>(reader: &mut R) -> Result<Option<Self::Output>, PgsError>
    where
        R: BufRead + Seek;
}

/// Decoder for `PGS` who provide only the times of subtitles.
pub struct DecodeTimeOnly;
impl PgsDecoder for DecodeTimeOnly {
    type Output = TimeSpan;

    fn parse_next<R>(reader: &mut R) -> Result<Option<Self::Output>, PgsError>
    where
        R: BufRead + Seek,
    {
        let mut start_time = None;
        let mut subtitle = None;

        while let Some(segment_header) = {
            if subtitle.is_some() {
                None
            } else {
                read_header(reader).transpose()
            }
        } {
            let seg_header = segment_header?;
            match seg_header.type_code() {
                SegmentTypeCode::End => {
                    let time = TimePoint::from_msecs(i64::from(seg_header.presentation_time()));

                    if let Some(start_time) = start_time {
                        subtitle = Some(TimeSpan::new(start_time, time))
                    } else {
                        start_time = Some(time);
                    }
                }
                _ => {
                    // Not managed for now
                }
            }
        }

        Ok(subtitle)
    }
}
