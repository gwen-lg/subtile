//! Module for `Image` manipulation.
mod utils;

pub use utils::{dump_images, DumpError};

use crate::content::Area;

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
