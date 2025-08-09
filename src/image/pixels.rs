use image::{Luma, LumaA, Primitive};
use std::borrow::Borrow;

/// Convert Pixel from [`LumaA`] to [`Luma`] to remove alpha.
///
/// This function is useful to prepare image for `ocr`.
/// If the alpha and luma value of the pixel is greater than or equal to threshold values,
/// the output is [`Primitive::DEFAULT_MIN_VALUE`] (equivalent to black).
/// Otherwise, the returned value is [`Primitive::DEFAULT_MAX_VALUE`] (equivalent to white).
///
/// * `A`: alpha threshold
/// * `L` : luma threshold
///
/// # Panics
/// Will panic if `P`(Primitive) is not initializable from value `L` and `A`.
pub fn luma_a_to_luma<In, P, const A: u8, const L: u8>(luma: In) -> Luma<P>
where
    In: Borrow<LumaA<P>>,
    P: Primitive,
{
    let luma = luma.borrow();
    let luminance = luma[0]; //0 : Luminance idx
    let alpha = luma[1]; //1 : Alpha idx

    if alpha >= P::from(A).unwrap() && luminance >= P::from(L).unwrap() {
        Luma([P::DEFAULT_MIN_VALUE])
    } else {
        Luma([P::DEFAULT_MAX_VALUE])
    }
}

/// Create and return a closure than convert a Pixel from [`LumaA`] to [`Luma`].
///
/// The closure apply threasold value from function parameters. If the alpha and luma value
/// of the pixel is greater than or equal to threshold values, the output is [`Primitive::DEFAULT_MIN_VALUE`] (equivalent to black).
/// Otherwise, the returned value is [`Primitive::DEFAULT_MAX_VALUE`] (equivalent to white).
///
/// * `alpha_t`: alpha threshold
/// * `luma_t`: luma threshold
pub fn luma_a_to_luma_convertor<P, In>(alpha_t: P, luma_t: P) -> impl Fn(In) -> Luma<P>
where
    P: Primitive,
    In: Borrow<LumaA<P>>,
{
    move |luma| {
        let luma = luma.borrow();
        let luminance = luma[0]; //0 : Luminance idx
        let alpha = luma[1]; //1 : Alpha idx
        if alpha >= alpha_t && luminance >= luma_t {
            Luma([P::DEFAULT_MIN_VALUE])
        } else {
            Luma([P::DEFAULT_MAX_VALUE])
        }
    }
}
