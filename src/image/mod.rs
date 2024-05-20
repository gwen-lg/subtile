//! Module for `Image` manipulation.
mod utils;

pub use utils::{dump_images, DumpError};

/// Define access to Size of an Image. Used for Subtitle content.
pub trait ImageSize {
    /// access to width of the image
    fn width(&self) -> u32;
    /// access to height of the image
    fn height(&self) -> u32;
}
