//! Module for subtitle content utils
mod area;
mod size;

pub use area::{Area, AreaValues};
pub use size::Size;

use thiserror::Error;

/// Error for content
#[derive(Debug, Error)]
pub enum ContentError {
    /// Indicate an invalid bounding box Area
    /// Example: If at least one coordinate value of second point are inferior of first point.
    #[error("Invalid bounding box for Area")]
    InvalidAreaBounding,
}
