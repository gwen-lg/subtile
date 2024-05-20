use super::VobSubIndexedImage;

/// The trait `VobSubDecoder` define the behavior to output data from `VobSub` parsing
pub trait VobSubDecoder {
    type Output;

    fn from_data(
        start_time: f64,
        end_time: Option<f64>,
        force: bool,
        image: VobSubIndexedImage,
    ) -> Self::Output;
}
