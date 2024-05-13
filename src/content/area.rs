use super::Size;
use crate::SubError;

/// Location at which to display the subtitle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaValues {
    /// min `x` coordinate value
    pub x1: u16,
    /// min `y` coordinate value
    pub y1: u16,
    /// max `x` coordinate value
    pub x2: u16,
    /// max `y` coordinate value
    pub y2: u16,
}

/// Location at which to display the subtitle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Area(AreaValues);

impl Area {
    /// The leftmost edge of the subtitle.
    #[must_use]
    pub fn left(&self) -> u16 {
        self.0.x1
    }

    /// The rightmost edge of the subtitle.
    #[must_use]
    pub fn top(&self) -> u16 {
        self.0.y1
    }

    /// The width of the subtitle.
    #[must_use]
    pub fn width(&self) -> u16 {
        self.0.x2 + 1 - self.0.x1
    }

    /// The height of the subtitle.
    #[must_use]
    pub fn height(&self) -> u16 {
        self.0.y2 + 1 - self.0.y1
    }

    /// The size of the subtitle.
    #[must_use]
    pub fn size(&self) -> Size {
        Size {
            w: cast::usize(self.width()),
            h: cast::usize(self.height()),
        }
    }
}

impl TryFrom<AreaValues> for Area {
    type Error = SubError;

    fn try_from(coords_value: AreaValues) -> Result<Self, Self::Error> {
        // Check for weird bounding boxes.  Ideally we
        // would do this while parsing, but I can't
        // figure out how to get nom to do what I want.
        // Later on, we assume that all bounding boxes
        // have non-negative width and height and we'll
        // crash if they don't.
        if coords_value.x2 <= coords_value.x1 || coords_value.y2 <= coords_value.y1 {
            Err(SubError::Parse("invalid bounding box".into()))
        } else {
            Ok(Self(coords_value))
        }
    }
}
