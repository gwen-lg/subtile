use crate::time::{TimePoint, TimeSpan};
use std::io::{BufRead, Seek};

use super::{
    ods, pds,
    pgs_image::RleEncodedImage,
    segment::{read_header, skip_segment, SegmentTypeCode},
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

/// Decoder for `PGS` who provide the times and images of the subtitles.
pub struct DecodeTimeImage {}
impl PgsDecoder for DecodeTimeImage {
    type Output = (TimeSpan, RleEncodedImage);

    fn parse_next<R>(reader: &mut R) -> Result<Option<Self::Output>, PgsError>
    where
        R: BufRead + Seek,
    {
        let mut start_time = None;
        let mut subtitle = None;
        let mut palette = None;
        let mut image = None;

        while let Some(segment) = {
            if subtitle.is_some() {
                None
            } else {
                read_header(reader).transpose()
            }
        } {
            let header = segment?;
            match header.type_code() {
                SegmentTypeCode::Pds => {
                    let seg_size = header.size() as usize;
                    let pds = pds::read(reader, seg_size)?;
                    palette = Some(pds.palette);
                }
                SegmentTypeCode::Ods => {
                    let seg_size = header.size() as usize;
                    let ods = ods::read(reader, seg_size)?;

                    let palette = palette.take().ok_or_else(|| PgsError::MissingPalette)?;
                    image = Some(RleEncodedImage::new(
                        ods.width,
                        ods.height,
                        palette,
                        ods.object_data,
                    ))
                }
                SegmentTypeCode::End => {
                    let time = TimePoint::from_msecs(i64::from(header.presentation_time()));

                    if let Some(start_time) = start_time {
                        let times = TimeSpan::new(start_time, time);

                        let image = image.take().ok_or_else(|| PgsError::MissingImage)?;
                        subtitle = Some((times, image));
                    } else {
                        start_time = Some(time);
                    }
                }
                _ => {
                    // Segment not taken into account are skipped
                    skip_segment(reader, &header)?;
                }
            };
        }

        Ok(subtitle)
    }
}
