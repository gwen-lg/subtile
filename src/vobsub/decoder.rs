use crate::content::Area;

/// The trait `VobSubDecoder` define the behavior to output data from `VobSub` parsing
pub trait VobSubDecoder {
    type Output;

    fn from_data(
        start_time: f64,
        end_time: Option<f64>,
        force: bool,
        area: Area,
        palette: [u8; 4],
        alpha: [u8; 4],
        raw_image: Vec<u8>,
    ) -> Self::Output;
}
