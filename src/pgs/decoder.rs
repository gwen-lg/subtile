use crate::time::{TimePoint, TimeSpan};
use std::io::{BufRead, Seek};

use super::{
    PgsError,
    ods::{self, ObjectDefinitionSegment},
    pds,
    pgs_image::RleEncodedImage,
    segment::{SegmentTypeCode, read_header, skip_segment},
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

        while let Some(seg_header) = {
            if subtitle.is_some() {
                None
            } else {
                read_header(reader)?
            }
        } {
            match seg_header.type_code() {
                SegmentTypeCode::End => {
                    let time = TimePoint::from_msecs(i64::from(seg_header.presentation_time()));

                    if let Some(start_time) = start_time {
                        subtitle = Some(TimeSpan::new(start_time, time));
                    } else {
                        start_time = Some(time);
                    }
                }
                SegmentTypeCode::Pds
                | SegmentTypeCode::Ods
                | SegmentTypeCode::Pcs
                | SegmentTypeCode::Wds => {
                    // Segment content are not taken into account, skipped
                    skip_segment(reader, &seg_header)?;
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
        let mut prev_ods = None;

        while let Some(seg_header) = {
            if subtitle.is_some() {
                None
            } else {
                read_header(reader)?
            }
        } {
            match seg_header.type_code() {
                SegmentTypeCode::Pds => {
                    let seg_size = seg_header.size() as usize;
                    let pds = pds::read(reader, seg_size)?;
                    palette = Some(pds.palette);
                }
                SegmentTypeCode::Ods => {
                    let seg_size = seg_header.size() as usize;
                    let ods = ods::read(reader, seg_size, prev_ods.take())?;

                    // If data are complete, construct `image` from palette and image data
                    // otherwise, keep read data to complete it with data from following segment.
                    if let ObjectDefinitionSegment::Complete(ods) = ods {
                        let palette = palette.take().ok_or(PgsError::MissingPalette)?;
                        image = Some(RleEncodedImage::new(
                            ods.width,
                            ods.height,
                            palette,
                            ods.object_data,
                        ));
                    } else {
                        prev_ods = Some(ods);
                    }
                }
                SegmentTypeCode::End => {
                    let time = TimePoint::from_msecs(i64::from(seg_header.presentation_time()));

                    if let Some(start_time) = start_time {
                        let times = TimeSpan::new(start_time, time);

                        let image = image.take().ok_or(PgsError::MissingImage)?;
                        subtitle = Some((times, image));
                    } else {
                        start_time = Some(time);
                    }
                }
                SegmentTypeCode::Pcs | SegmentTypeCode::Wds => {
                    // Segment not taken into account are skipped
                    skip_segment(reader, &seg_header)?;
                }
            }
        }

        assert!(palette.is_none()); // palette should be transferred into image before get out of the function.
        assert!(prev_ods.is_none()); // Ods data should be converted into image before get out of the function.
        Ok(subtitle)
    }
}
