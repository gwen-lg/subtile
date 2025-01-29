//! Module for `Image` manipulation.
mod pixels;
mod utils;

// Re-export some useful image types.
pub use image::{GrayImage, Luma};
pub use pixels::{luma_a_to_luma, luma_a_to_luma_convertor};
pub use utils::{dump_images, DumpError};

use crate::content::Area;
use image::{ImageBuffer, Pixel};

/// Define access to Size of an Image. Used for Subtitle content.
pub trait ImageSize {
    /// access to width of the image
    fn width(&self) -> u32;
    /// access to height of the image
    fn height(&self) -> u32;
}

/// define access to Area of an Image. Used for Subtitle content.
pub trait ImageArea {
    ///access to area of the image
    fn area(&self) -> Area;
}

// Implement ImageSize for all type than implement ImageArea
impl<U> ImageSize for U
where
    U: ImageArea,
{
    fn width(&self) -> u32 {
        u32::from(self.area().width())
    }
    fn height(&self) -> u32 {
        u32::from(self.area().height())
    }
}

/// define the behavior of generate a `ImageBuffer` from a `self`
pub trait ToImage {
    /// Define the format of Sub-pixel of output
    type Pixel: Pixel<Subpixel = u8>;

    /// define the method to generate the image
    fn to_image(&self) -> ImageBuffer<Self::Pixel, Vec<u8>>;
}

/// Options for image generation.
#[derive(Debug, Clone, Copy)]
pub struct ToOcrImageOpt {
    /// Number of border pixels to add on the input image
    pub border: u32,
    /// Color of the text
    pub text_color: Luma<u8>,
    /// Color of the background
    pub background_color: Luma<u8>,
}

// Implement [`Default`] for [`ToOcrImageOpt`] with a border of 5 pixel
// and colors black for text and white for background.
impl Default for ToOcrImageOpt {
    fn default() -> Self {
        Self {
            border: 5,
            text_color: Luma([0]),
            background_color: Luma([255]),
        }
    }
}

/// Generate a `GrayImage` adapted for `OCR` from self.
pub trait ToOcrImage {
    /// Generate the image for `OCR` in `GrayImage` format.
    fn image(&self, opt: &ToOcrImageOpt) -> GrayImage;
}
