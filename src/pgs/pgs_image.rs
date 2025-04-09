use super::pds::{Palette, PaletteEntry};
use crate::image::{ImageSize, ToImage, ToOcrImage, ToOcrImageOpt};
use image::{ImageBuffer, Luma, LumaA, Pixel, Primitive};
use std::io::{ErrorKind, Read as _};

/// Define a type of `fn` who covert pixel from `PaletteEntry` to a target color type.
type PixelConversion<TargetColor> = fn(&PaletteEntry) -> TargetColor;

/// Store Image data directly from `PGS`.
#[derive(Clone)]
pub struct RleEncodedImage {
    width: u16,
    height: u16,
    palette: Palette,
    raw: Vec<u8>,
}

impl RleEncodedImage {
    /// Create a `RleEncodedImage` from [`SupParser`]
    ///
    /// [`SupParser`]: super::sup::SupParser
    #[must_use]
    pub const fn new(width: u16, height: u16, palette: Palette, raw: Vec<u8>) -> Self {
        Self {
            width,
            height,
            palette,
            raw,
        }
    }

    /// Iterate on image pixels converted with a specified function.
    pub fn pixels<D: Primitive>(
        &self,
        convert: PixelConversion<LumaA<D>>,
    ) -> RlePixelIterator<'_, LumaA<D>> {
        RlePixelIterator {
            rle_image: self,
            raw_data: &self.raw,
            current_color: LumaA([D::DEFAULT_MIN_VALUE, D::DEFAULT_MAX_VALUE]),
            default_color: LumaA([D::DEFAULT_MAX_VALUE, D::DEFAULT_MIN_VALUE]), // Default: white + transparent
            nb_remaining_pixels: 0,
            convert,
        }
    }
}

impl ImageSize for RleEncodedImage {
    fn width(&self) -> u32 {
        u32::from(self.width)
    }
    fn height(&self) -> u32 {
        u32::from(self.height)
    }
}

impl<'a> RleEncodedImage {
    /// Return and iterator over Pixels of the image.
    #[must_use]
    pub fn iter(&'a self) -> RlePixelIterator<'a, LumaA<u8>> {
        self.into_iter()
    }
}

/// Create an iterator over [`RleEncodedImage`] pixels.
impl<'a> IntoIterator for &'a RleEncodedImage {
    type Item = LumaA<u8>;
    type IntoIter = RlePixelIterator<'a, LumaA<u8>>;

    fn into_iter(self) -> Self::IntoIter {
        RlePixelIterator {
            rle_image: self,
            raw_data: &self.raw,
            current_color: LumaA([
                <u8 as Primitive>::DEFAULT_MIN_VALUE,
                <u8 as Primitive>::DEFAULT_MAX_VALUE,
            ]), // setup to luma min (black), alpha max (opaque)
            default_color: LumaA([
                <u8 as Primitive>::DEFAULT_MAX_VALUE,
                <u8 as Primitive>::DEFAULT_MIN_VALUE,
            ]), // Default: white + transparent
            nb_remaining_pixels: 0,
            convert: pe_to_luma_a,
        }
    }
}

/// Convert a [`PaletteEntry`] to a `LumaA`<P>
fn pe_to_luma_a<P: Primitive>(input: &PaletteEntry) -> LumaA<P> {
    let luminance = P::from(input.luminance).unwrap();
    let alpha = P::from(input.transparency).unwrap();
    LumaA([luminance, alpha])
}

/// Pass-through pixel to keep all information.
#[must_use]
pub const fn pixel_pass_through(pix: LumaA<u8>) -> LumaA<u8> {
    pix
}

/// This struct implement [`ToImage`] to generate an `ImageBuffer` from
/// a [`RleEncodedImage`] and a pixel conversion function.
pub struct RleToImage<'a, P, C>
where
    P: Pixel<Subpixel = u8>,
    C: Fn(LumaA<u8>) -> P,
{
    rle_image: &'a RleEncodedImage,
    conv_fn: C,
}

impl<'a, P, C> RleToImage<'a, P, C>
where
    P: Pixel<Subpixel = u8>,
    C: Fn(LumaA<u8>) -> P,
{
    /// Create a struct to generate an image from [`RleEncodedImage`]
    pub const fn new(rle_image: &'a RleEncodedImage, conv_fn: C) -> Self {
        Self { rle_image, conv_fn }
    }
}

impl<P, C> ToImage for RleToImage<'_, P, C>
where
    P: Pixel<Subpixel = u8>,
    C: Fn(LumaA<u8>) -> P,
{
    type Pixel = P;

    #[profiling::function]
    fn to_image(&self) -> ImageBuffer<P, Vec<u8>>
    where
        P: Pixel<Subpixel = u8>,
    {
        let width = self.rle_image.width();
        let height = self.rle_image.height();
        let pixel_iter = self.rle_image.into_iter();

        let buf_size = (width * height) as usize * P::CHANNEL_COUNT as usize;
        let mut buf = Vec::with_capacity(buf_size);
        pixel_iter
            .map(|p| (self.conv_fn)(p))
            .for_each(|p| buf.extend_from_slice(p.channels()));

        ImageBuffer::<P, Vec<u8>>::from_vec(width, height, buf)
            .expect("Failed to create image buffer")
    }
}

/// Implement [`ToOcrImage`] from [`RleEncodedImage`]
impl<C> ToOcrImage for RleToImage<'_, Luma<u8>, C>
where
    C: Fn(LumaA<u8>) -> Luma<u8>,
{
    #[profiling::function]
    fn image(&self, opt: &ToOcrImageOpt) -> image::GrayImage {
        let width = self.rle_image.width();
        let height = self.rle_image.height();
        let border = opt.border;

        let raw_pixels = self.rle_image.into_iter().collect::<Vec<_>>();

        ImageBuffer::from_fn(width + border * 2, height + border * 2, |x, y| {
            if x < border || x >= width + border || y < border || y >= height + border {
                opt.background_color
            } else {
                let offset = (y - border) * width + (x - border);
                let pixel = raw_pixels[offset as usize];
                (self.conv_fn)(pixel)
            }
        })
    }
}

/// struct to iterate on pixel of an `Rle` image.
pub struct RlePixelIterator<'a, C> {
    rle_image: &'a RleEncodedImage,
    raw_data: &'a [u8],
    current_color: C,
    default_color: C,
    nb_remaining_pixels: u16,
    convert: PixelConversion<C>,
}

/// Allow iterate over pixels of image encoded in `Rle`.
impl<Pix, Sub> Iterator for RlePixelIterator<'_, Pix>
where
    Sub: Primitive,
    Pix: Copy + Pixel<Subpixel = Sub>,
{
    type Item = Pix;

    fn next(&mut self) -> Option<Self::Item> {
        if self.nb_remaining_pixels > 0 {
            self.nb_remaining_pixels -= 1;
            Some(self.current_color)
        } else if let Some((color_id, nb_pixel)) = self.read_next_pixel() {
            let color = if let Some(color) = self.rle_image.palette.get(color_id) {
                (self.convert)(color)
            } else {
                // If color_id is not present in palette, return default value
                self.default_color
            };

            self.current_color = color;
            self.nb_remaining_pixels = nb_pixel - 1;
            Some(self.current_color)
        } else {
            None // End of pixels
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let nb_pixels = (self.rle_image.width() * self.rle_image.height()) as usize;
        (nb_pixels, Some(nb_pixels))
    }
}

impl<Pix, Sub> ExactSizeIterator for RlePixelIterator<'_, Pix>
where
    Sub: Primitive,
    Pix: Copy + Pixel<Subpixel = Sub>,
{
}

impl<C> RlePixelIterator<'_, C> {
    /// Read next pixel info(color and number of instance).
    fn read_next_pixel(&mut self) -> Option<(u8 /*color */, u16 /*nb_pixels*/)> {
        const MARKER: u8 = 0;
        const COLOR_0: u8 = 0;
        loop {
            let mut color: [u8; 1] = [0; 1];
            let res = self.raw_data.read_exact(&mut color);
            if let Err(err) = res {
                if err.kind() == ErrorKind::UnexpectedEof {
                    return None;
                }
            }

            let next = color[0];
            if next == MARKER {
                let mut l2 = [0; 1];
                self.raw_data.read_exact(&mut l2).unwrap();

                if l2[0] == MARKER {
                    //break; // End of line
                } else {
                    let byte = l2[0];
                    let nb_pixels = match CountMarker::from(byte) {
                        CountMarker::Long => {
                            let mut l3 = [0; 1];
                            self.raw_data.read_exact(&mut l3).unwrap();
                            let count_bytes = [byte & 0b0011_1111, l3[0]];
                            u16::from_be_bytes(count_bytes)
                        }
                        CountMarker::Short => {
                            let count_bits = byte & 0b0011_1111;
                            u16::from(u8::from_be(count_bits))
                        }
                    };

                    let color_marker = ColorMarker::from(byte);
                    let color = match color_marker {
                        ColorMarker::Color0 => COLOR_0,
                        ColorMarker::ColorN => {
                            let mut color = [0; 1];
                            self.raw_data.read_exact(&mut color).unwrap();
                            color[0]
                        }
                    };

                    return Some((color, nb_pixels));
                }
            } else {
                return Some((next, 1));
            }
        }
    }
}

/// Decode the color marker.
enum ColorMarker {
    /// color 0 : black
    Color0,
    /// color N : color define in code
    ColorN,
}
impl From<u8> for ColorMarker {
    fn from(value: u8) -> Self {
        if (value & 0b1000_0000) > 0 {
            Self::ColorN
        } else {
            Self::Color0
        }
    }
}

/// Decode the pixels count marcker.
enum CountMarker {
    /// the number of pixels is between 1 and 63
    Short,
    /// the number of pixels is between 64 and 16383
    Long,
}
impl From<u8> for CountMarker {
    fn from(value: u8) -> Self {
        if (value & 0b0100_0000) > 0 {
            Self::Long
        } else {
            Self::Short
        }
    }
}
