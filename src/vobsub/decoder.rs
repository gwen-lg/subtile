use super::{img::VobSubRleImage, VobSubIndexedImage};
use crate::time::{TimePoint, TimeSpan};

/// The default length of a subtitle if no end time is provided and no
/// subtitle follows immediately after.
const DEFAULT_SUBTITLE_LENGTH: f64 = 5.0;

/// The trait `VobSubDecoder` define the behavior to output data from `VobSub` parsing.
/// This trait is used by [`VobSubParser`] to allow various decoding of parsing data.
pub trait VobSubDecoder<'a> {
    type Output;

    fn from_data(
        start_time: f64,
        end_time: Option<f64>,
        force: bool,
        image: VobSubRleImage<'a>,
    ) -> Self::Output;
}

/// Implement creation of a tuple of [`TimeSpan`] and [`VobSubIndexedImage`] from parsing.
impl<'a> VobSubDecoder<'a> for (TimeSpan, VobSubIndexedImage) {
    type Output = Self;

    fn from_data(
        start_time: f64,
        end_time: Option<f64>,
        _force: bool,
        rle_image: VobSubRleImage<'a>,
    ) -> Self::Output {
        (
            TimeSpan::new(
                TimePoint::from_secs(start_time),
                TimePoint::from_secs(end_time.unwrap_or(DEFAULT_SUBTITLE_LENGTH)),
            ),
            VobSubIndexedImage::from(rle_image),
        )
    }
}
