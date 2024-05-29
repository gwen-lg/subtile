use super::img::VobSubRleImage;

/// The trait `VobSubDecoder` define the behavior to output data from `VobSub` parsing
pub trait VobSubDecoder<'a> {
    type Output;

    fn from_data(
        start_time: f64,
        end_time: Option<f64>,
        force: bool,
        image: VobSubRleImage<'a>,
    ) -> Self::Output;
}
