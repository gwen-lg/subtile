//! Module for `Image` manipulation.
mod utils;

pub use utils::{dump_images, DumpError};

use crate::content::Area;
use image::{GrayImage, ImageBuffer, Luma, Pixel};
use std::ops::Deref;

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
    /// Define the format of Subpixel of output
    type Pixel: Pixel<Subpixel = u8>;
    ///TODO: Define the container type
    // type Container: Deref<Target = [<Pixel as image::Pixel>::Subpixel]>;

    /// define the method to generate the image
    fn to_image(&self) -> ImageBuffer<Self::Pixel, Vec<u8>>;
}

/// define an operation to generate a
pub trait ToImage2 {
    /// Define the subpixel format of the output image.
    type Subpixel;
    /// Define the container type of pixels of the output image.
    type Container;
    /// Define the format of pixels of output image.
    type Pixel: Pixel<Subpixel = Self::Subpixel>;
    ///TODO: Define the container type
    // type Container: Deref<Target = [<Pixel as image::Pixel>::Subpixel]>;

    /// define the method to generate the image
    fn to_image(&self) -> ImageBuffer<Self::Pixel, Self::Container>;
}

// test
impl<P, C, S> ToImage2 for ImageBuffer<P, C>
where
    P: Pixel<Subpixel = S>,
    C: Clone + Deref<Target = [P::Subpixel]>,
{
    type Subpixel = S;
    type Container = C;
    type Pixel = P;

    fn to_image(&self) -> ImageBuffer<Self::Pixel, Self::Container> {
        self.clone()
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
