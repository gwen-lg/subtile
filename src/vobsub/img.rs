//! Run-length encoded image format for subtitles.

use core::fmt::{self, Debug};
use image::{ImageBuffer, Luma, Pixel, Rgb, Rgba};
use iter_fixed::IntoIteratorFixed;
use log::trace;
use nom::{
    bits::complete::{tag as tag_bits, take as take_bits},
    branch::alt,
    combinator::value,
    sequence::{preceded, Tuple},
    IResult,
};
use thiserror::Error;

use super::{IResultExt, NomError, VobSubError};
use crate::{
    content::{Area, Size},
    image::{ImageArea, ImageSize, ToImage, ToOcrImage, ToOcrImageOpt},
    util::BytesFormatter,
};

/// Errors of `vobsub` img management.
#[derive(Error, Debug)]
pub enum Error {
    /// If there is more data to write than the space in output.
    #[error("output parameter is too small (size:{output_size}) for write scanline data (size:{data_size})")]
    ToSmallOutput {
        data_size: usize,
        output_size: usize,
    },

    /// If index value is bigger than the image width.
    #[error("Scan line is longer than image width: [{x},{width}]")]
    ScanLineLongerThanWidth { x: usize, width: usize },

    /// Forward scan line parsing error.
    #[error("Parsing scan line failed")]
    ScanLineParsing(#[source] NomError),
}

pub struct VobSubRleImage<'a> {
    area: Area,
    palette: [u8; 4],
    alpha: [u8; 4],
    image_data: VobSubRleImageData<'a>,
}
impl<'a> VobSubRleImage<'a> {
    pub const fn new(
        area: Area,
        palette: [u8; 4],
        alpha: [u8; 4],
        image_data: VobSubRleImageData<'a>,
    ) -> Self {
        Self {
            area,
            palette,
            alpha,
            image_data,
        }
    }

    pub fn size(&self) -> Size {
        self.area.size()
    }
    pub const fn palette(&self) -> &[u8; 4] {
        &self.palette
    }
    pub const fn alpha(&self) -> &[u8; 4] {
        &self.alpha
    }
    pub const fn raw_data(&self) -> &VobSubRleImageData<'a> {
        &self.image_data
    }
}

impl ImageArea for VobSubRleImage<'_> {
    fn area(&self) -> Area {
        self.area
    }
}

/// Handle `VobSub` `Rle` image data in one struct.
pub struct VobSubRleImageData<'a> {
    data: [&'a [u8]; 2],
}
impl<'a> VobSubRleImageData<'a> {
    pub fn new(raw_data: &'a [u8], rle_offsets: [u16; 2], end: usize) -> Result<Self, VobSubError> {
        // We know the starting points of each set of scan lines, but we don't
        // really know where they end, because encoders like to reuse bytes
        // that they're already using for something else.  For example, the
        // last few bytes of the first set of scan lines may overlap with the
        // first bytes of the second set of scanlines, and the last bytes of
        // the second set of scan lines may overlap with the start of the
        // control sequence.  For now, we limit it to the first two bytes of
        // the control packet, which are usually `[0x00, 0x00]`.  (We might
        // actually want to remove `end` entirely here and allow the scan lines
        // to go to the end of the packet, but I've never seen that in
        // practice.)
        let start_0 = usize::from(rle_offsets[0]);
        let start_1 = usize::from(rle_offsets[1]);

        if start_0 > start_1 || start_1 > end {
            Err(VobSubError::InvalidScanLineOffsets {
                start_0,
                start_1,
                end,
            })
        } else {
            Ok(Self {
                data: [&raw_data[start_0..end], &raw_data[start_1..end]],
            })
        }
    }
}

/// A run-length encoded value.
#[derive(Debug)]
struct Rle {
    /// The number of times to repeat this value.  A value of 0 indicates that
    /// we should fill to the end of the line.
    cnt: u16,
    /// The value to repeat.  This is 2 bits wide.
    val: u8,
}

/// Parse the count for a `Rle`.
fn count(input: (&[u8], usize)) -> IResult<(&[u8], usize), u16> {
    // Fill to end of line.
    let end_of_line = value(0, tag_bits(0, 14u16));
    // Count for 4-nibble RLE.
    let count4 = preceded(tag_bits(0, 6u8), take_bits(8u16));
    // Count for 3-nibble RLE.
    let count3 = preceded(tag_bits(0, 4u8), take_bits(6u16));
    // Count for 2-nibble RLE.
    let count2 = preceded(tag_bits(0, 2u8), take_bits(4u16));
    // Count for 1-nibble RLE.
    let count1 = take_bits(2u16);
    alt((end_of_line, count4, count3, count2, count1))(input)
}

/// Parse an `Rle`.
fn rle(input: (&[u8], usize)) -> IResult<(&[u8], usize), Rle> {
    let take_val = take_bits(2u8);
    let (input, (cnt, val)) = (count, take_val).parse(input)?;
    Ok((input, Rle { cnt, val }))
}

/// Decompress the scan-line `input` into `output`, returning the number of
/// input bytes consumed.
fn scan_line(input: &[u8], output: &mut [u8]) -> Result<usize, Error> {
    trace!("scan line starting with {:?}", BytesFormatter(input));
    let width = output.len();
    let mut x = 0;
    let mut pos = (input, 0);
    while x < width {
        let (new_pos, run) = rle(pos).to_result().map_err(Error::ScanLineParsing)?;

        //trace!("RLE: {:?}", &run);
        pos = new_pos;
        let count = if run.cnt == 0 {
            width - x
        } else {
            usize::from(run.cnt)
        };
        if x + count > output.len() {
            return Err(Error::ToSmallOutput {
                data_size: x + count,
                output_size: output.len(),
            });
        }
        output[x..x + count].fill(run.val);
        x += count;
    }
    if x > width {
        return Err(Error::ScanLineLongerThanWidth { x, width });
    }
    // Round up to the next full byte.
    if pos.1 > 0 {
        pos = (&pos.0[1..], 0);
    }
    Ok(input.len() - pos.0.len())
}

/// Decompress a run-length encoded image, and return a vector in row-major
/// order, starting at the upper-left and scanning right and down, with one
/// byte for each 2-bit value.
#[profiling::function]
pub fn decompress(size: Size, data: &VobSubRleImageData) -> Result<Vec<u8>, Error> {
    trace!(
        "decompressing image {:?}, max: [0x{:x}, 0x{:x}]",
        &size,
        data.data[0].len(),
        data.data[1].len()
    );
    let mut img = vec![0; size.w * size.h];
    let mut offsets = [0; 2];
    for y in 0..size.h {
        let odd = y % 2;
        trace!("line {:?}, offset 0x{:x}", y, offsets[odd]);
        let consumed = scan_line(
            &data.data[odd][offsets[odd]..],
            &mut img[y * size.w..(y + 1) * size.w],
        )?;
        offsets[odd] += consumed;
    }
    // TODO: Warn if we didn't consume everything.
    Ok(img)
}

/// Manage image data from `VobSub` file.
#[derive(Clone, PartialEq, Eq)]
pub struct VobSubIndexedImage {
    /// Coordinates at which to display the subtitle.
    area: Area,
    /// Map each of the 4 colors in this subtitle to a 4-bit palette.
    palette: [u8; 4],
    /// Map each of the 4 colors in this subtitle to 4 bits of alpha
    /// channel data.
    //TODO: encapsulate in dedicated type for avoiding error with palette
    alpha: [u8; 4],
    /// Our decompressed image, stored with 2 bits per byte in row-major
    /// order, that can be used as indices into `palette` and `alpha`.
    raw_image: Vec<u8>,
}
impl VobSubIndexedImage {
    /// Create a new `VobSubImage`
    #[must_use]
    pub fn new(area: Area, palette: [u8; 4], alpha: [u8; 4], raw_image: Vec<u8>) -> Self {
        Self {
            area,
            palette,
            alpha,
            raw_image,
        }
    }

    /// Access to palette data
    #[must_use]
    pub const fn palette(&self) -> &[u8; 4] {
        &self.palette
    }

    /// Access to alpha data
    #[must_use]
    pub const fn alpha(&self) -> &[u8; 4] {
        &self.alpha
    }

    /// Access to pixel raw data of the image
    #[must_use]
    pub fn raw_image(&self) -> &[u8] {
        self.raw_image.as_slice()
    }
}

impl fmt::Debug for VobSubIndexedImage {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("VobSub Image")
            .field("area", &self.area)
            .field("palette", &self.palette)
            .field("alpha", &self.alpha)
            .finish_non_exhaustive()
    }
}

impl ImageArea for VobSubIndexedImage {
    fn area(&self) -> Area {
        self.area
    }
}

impl From<VobSubRleImage<'_>> for VobSubIndexedImage {
    fn from(rle_image: VobSubRleImage) -> Self {
        let decompressed_image = decompress(rle_image.size(), rle_image.raw_data()).unwrap();
        Self::new(
            rle_image.area(),
            *rle_image.palette(),
            *rle_image.alpha(),
            decompressed_image,
        )
    }
}

/// convert rbg + alpha to `Rgba`
#[must_use]
pub fn conv_to_rgba(color: Rgb<u8>, alpha: u8) -> Rgba<u8> {
    Rgba([
        color.channels()[0],
        color.channels()[1],
        color.channels()[2],
        alpha,
    ])
}

/// This struct implement [`ToImage`] to generate an `ImageBuffer` from
/// a [`VobSubIndexedImage`], a palette and a pixel conversion function.
pub struct VobSubToImage<'a, I, P>
where
    P: Pixel<Subpixel = u8>,
{
    indexed_img: &'a VobSubIndexedImage,
    palette: &'a [I; 16],
    conv_fn: fn(I, u8) -> P,
}

impl<'a, I, P> VobSubToImage<'a, I, P>
where
    P: Pixel<Subpixel = u8>,
{
    /// Create a `VobSub` image converter from a [`VobSubIndexedImage`], a `palette` and
    /// a pixel conversion function.
    #[must_use]
    pub fn new(img: &'a VobSubIndexedImage, palette: &'a [I; 16], conv_fn: fn(I, u8) -> P) -> Self {
        Self {
            indexed_img: img,
            palette,
            conv_fn,
        }
    }

    fn compute_palette_color(&self, conv: fn(I, u8) -> P) -> [P; 4]
    where
        I: Clone,
        P: Pixel<Subpixel = u8>,
    {
        self.indexed_img
            .palette()
            .into_iter_fixed()
            .zip(self.indexed_img.alpha())
            .map(|(&palette_idx, &alpha)| (self.palette[palette_idx as usize].clone(), alpha))
            .map(|(luminance, alpha)| conv(luminance, alpha))
            .collect()
    }
}
impl<I, P> ToImage for VobSubToImage<'_, I, P>
where
    I: Clone,
    P: Pixel<Subpixel = u8>,
{
    type Pixel = P;

    #[profiling::function]
    fn to_image(&self) -> ImageBuffer<P, Vec<u8>>
    where
        P: Pixel<Subpixel = u8>,
    {
        let width = self.indexed_img.width();
        let height = self.indexed_img.height();
        let out_color_palette = self.compute_palette_color(self.conv_fn);

        let image = ImageBuffer::from_fn(width, height, |x, y| {
            let offset = y * width + x;
            let sub_palette_idx = self.indexed_img.raw_image()[offset as usize] as usize;
            out_color_palette[sub_palette_idx]
        });
        image
    }
}

/// A struct to convert [`VobSubIndexedImage`] to image for `OCR`
pub struct VobSubOcrImage<'a> {
    indexed_img: &'a VobSubIndexedImage,
    palette: &'a [f32; 16],
}

impl<'a> VobSubOcrImage<'a> {
    /// create the image converter.
    #[must_use]
    pub const fn new(indexed_img: &'a VobSubIndexedImage, palette: &'a [f32; 16]) -> Self {
        Self {
            indexed_img,
            palette,
        }
    }

    // Compute the output palette color
    fn compute_palette_color(&self, opt: ToOcrImageOpt) -> [Luma<u8>; 4] {
        self.indexed_img
            .palette()
            .into_iter_fixed()
            .zip(self.indexed_img.alpha())
            .map(|(&palette_idx, &alpha)| (self.palette[palette_idx as usize], alpha))
            .map(|(luminance, alpha)| {
                if alpha > 0 && luminance > 0. {
                    opt.text_color
                } else {
                    opt.background_color
                }
            })
            .collect()
    }
}

impl ToOcrImage for VobSubOcrImage<'_> {
    #[profiling::function]
    fn image(&self, opt: &ToOcrImageOpt) -> image::GrayImage {
        let width = self.indexed_img.width();
        let height = self.indexed_img.height();
        let border = opt.border;
        let out_color_palette = self.compute_palette_color(*opt);

        let image = ImageBuffer::from_fn(width + border * 2, height + border * 2, |x, y| {
            if x < border || x >= width + border || y < border || y >= height + border {
                opt.background_color
            } else {
                let offset = (y - border) * width + (x - border);
                let sub_palette_idx = self.indexed_img.raw_image()[offset as usize] as usize;
                out_color_palette[sub_palette_idx]
            }
        });
        image
    }
}
