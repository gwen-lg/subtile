//! Module for `Image` manipulation.
mod utils;

pub use utils::{dump_images, DumpError};

use crate::content::Area;
use image::{GrayImage, Luma};

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
