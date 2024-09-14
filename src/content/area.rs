use std::cmp::{max, min};

use super::{ContentError, Size};

/// Location at which to display the subtitle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
/// This is intended to manage values of pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Area(AreaValues);

impl Area {
    /// The leftmost edge of the subtitle.
    #[must_use]
    pub const fn left(&self) -> u16 {
        self.0.x1
    }

    /// The rightmost edge of the subtitle.
    #[must_use]
    pub const fn top(&self) -> u16 {
        self.0.y1
    }

    /// The width of the subtitle.
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.0.x2 + 1 - self.0.x1
    }

    /// The height of the subtitle.
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.0.y2 + 1 - self.0.y1
    }

    /// The size of the subtitle.
    #[must_use]
    pub fn size(&self) -> Size {
        Size {
            w: usize::from(self.width()),
            h: usize::from(self.height()),
        }
    }

    /// Indicate if the provided `Area` intersect with self.
    #[must_use]
    pub const fn intersect(&self, area: Self) -> bool {
        self.0.x1 <= area.0.x2
            && self.0.x2 >= area.0.x1
            && self.0.y1 <= area.0.y2
            && self.0.y2 >= area.0.y1
    }

    /// Indicate if the provided `Area` intersect on `y` axis with self.
    #[must_use]
    pub const fn intersect_y(&self, area: Self) -> bool {
        self.0.y1 <= area.0.y2 && self.0.y2 >= area.0.y1
    }

    /// Indicate if the provided `Area` is contained in self bounds.
    #[must_use]
    pub const fn contains(&self, area: Self) -> bool {
        self.0.x1 <= area.0.x1
            && self.0.x2 >= area.0.x2
            && self.0.y1 <= area.0.y1
            && self.0.y2 >= area.0.y2
    }

    /// Extend the area from an other `Area`
    pub fn extend(&mut self, area: Self) {
        self.0.x1 = min(self.0.x1, area.0.x1);
        self.0.y1 = min(self.0.y1, area.0.y1);
        self.0.x2 = max(self.0.x2, area.0.x2);
        self.0.y2 = max(self.0.y2, area.0.y2);
    }
}

impl TryFrom<AreaValues> for Area {
    type Error = ContentError;

    fn try_from(coords_value: AreaValues) -> Result<Self, Self::Error> {
        // Check for weird bounding boxes.  Ideally we
        // would do this while parsing, but I can't
        // figure out how to get nom to do what I want.
        // Later on, we assume that all bounding boxes
        // have non-negative width and height and we'll
        // crash if they don't.
        if coords_value.x2 <= coords_value.x1 || coords_value.y2 <= coords_value.y1 {
            Err(ContentError::InvalidAreaBounding)
        } else {
            Ok(Self(coords_value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Area, AreaValues};
    use crate::content::ContentError;

    const AREA_REF: AreaValues = AreaValues {
        x1: 10,
        y1: 10,
        x2: 20,
        y2: 20,
    };

    #[test]
    fn area_creation() {
        Area::try_from(AREA_REF).unwrap();

        let invalid_init = matches!(
            Area::try_from(AreaValues {
                x1: 20,
                y1: 10,
                x2: 10,
                y2: 20,
            }),
            Err(ContentError::InvalidAreaBounding)
        );
        assert!(invalid_init);
    }

    #[test]
    fn area_intersect() {
        let area_ref = Area::try_from(AREA_REF).unwrap();
        assert!(area_ref.intersect(area_ref));
        assert!(area_ref.intersect(Area(AreaValues {
            x1: 1,
            y1: 1,
            x2: 11,
            y2: 11,
        })));
        assert!(area_ref.intersect(Area(AreaValues {
            x1: 11,
            y1: 11,
            x2: 21,
            y2: 21,
        })));
        assert!(area_ref.intersect(Area(AreaValues {
            x1: 0,
            y1: 11,
            x2: 11,
            y2: 11,
        })));
        assert!(area_ref.intersect(Area(AreaValues {
            x1: 19,
            y1: 19,
            x2: 23,
            y2: 21,
        })));
        assert!(area_ref.intersect(Area(AreaValues {
            x1: 11,
            y1: 11,
            x2: 12,
            y2: 20,
        })));
        assert!(area_ref.intersect(Area(AreaValues {
            x1: 0,
            y1: 0,
            x2: 30,
            y2: 30,
        })));
        assert!(area_ref.intersect(Area(AreaValues {
            x1: 20,
            y1: 10,
            x2: 30,
            y2: 20,
        })));
    }

    #[test]
    const fn area_not_intersect() {
        let area_ref = Area(AREA_REF);
        assert!(!area_ref.intersect(Area(AreaValues {
            x1: 0,
            y1: 5,
            x2: 5,
            y2: 15,
        })));
        assert!(!area_ref.intersect(Area(AreaValues {
            x1: 11,
            y1: 21,
            x2: 12,
            y2: 22,
        })));
        assert!(!area_ref.intersect(Area(AreaValues {
            x1: 30,
            y1: 11,
            x2: 32,
            y2: 20,
        })));
        assert!(!area_ref.intersect(Area(AreaValues {
            x1: 11,
            y1: 21,
            x2: 19,
            y2: 22,
        })));
    }

    #[test]
    const fn area_intersect_y() {
        let area_ref = Area(AREA_REF);
        assert!(area_ref.intersect_y(area_ref));
        assert!(area_ref.intersect_y(Area(AreaValues {
            x1: 50,
            y1: 5,
            x2: 15,
            y2: 15,
        })));
        assert!(area_ref.intersect_y(Area(AreaValues {
            x1: 20,
            y1: 5,
            x2: 30,
            y2: 15,
        })));
        assert!(area_ref.intersect_y(Area(AreaValues {
            x1: 0,
            y1: 10,
            x2: 1,
            y2: 20,
        })));
        assert!(area_ref.intersect_y(Area(AreaValues {
            x1: 10,
            y1: 11,
            x2: 11,
            y2: 12,
        })));
        assert!(area_ref.intersect_y(Area(AreaValues {
            x1: 10,
            y1: 20,
            x2: 11,
            y2: 22,
        })));
    }

    #[test]
    const fn area_not_intersect_y() {
        let area_ref = Area(AREA_REF);

        assert!(!area_ref.intersect_y(Area(AreaValues {
            x1: 0,
            y1: 0,
            x2: 30,
            y2: 9,
        })));
        assert!(!area_ref.intersect_y(Area(AreaValues {
            x1: 0,
            y1: 21,
            x2: 30,
            y2: 25,
        })));
        assert!(!area_ref.intersect_y(Area(AreaValues {
            x1: 10,
            y1: 5,
            x2: 20,
            y2: 9,
        })));
    }

    #[test]
    const fn area_contain() {
        let area_ref = Area(AREA_REF);

        assert!(area_ref.contains(area_ref));
        assert!(area_ref.contains(Area(AreaValues {
            x1: 10,
            y1: 11,
            x2: 20,
            y2: 12,
        })));
        assert!(area_ref.contains(Area(AreaValues {
            x1: 11,
            y1: 11,
            x2: 19,
            y2: 19,
        })));
        assert!(area_ref.contains(Area(AreaValues {
            x1: 14,
            y1: 17,
            x2: 15,
            y2: 18,
        })));
    }

    #[test]
    const fn area_not_contain() {
        let area_ref = Area(AREA_REF);

        assert!(!area_ref.contains(Area(AreaValues {
            x1: 9,
            y1: 11,
            x2: 12,
            y2: 12,
        })));
        assert!(!area_ref.contains(Area(AreaValues {
            x1: 9,
            y1: 11,
            x2: 12,
            y2: 12,
        })));
        assert!(!area_ref.contains(Area(AreaValues {
            x1: 10,
            y1: 10,
            x2: 21,
            y2: 20,
        })));
        assert!(!area_ref.contains(Area(AreaValues {
            x1: 20,
            y1: 11,
            x2: 21,
            y2: 19,
        })));
    }

    #[test]
    fn area_extend() {
        let area_ref = Area(AREA_REF);

        assert!(
            {
                let mut new = area_ref;
                new.extend(Area(AreaValues {
                    x1: 10,
                    y1: 20,
                    x2: 20,
                    y2: 30,
                }));
                new
            } == Area(AreaValues {
                x1: 10,
                y1: 10,
                x2: 20,
                y2: 30
            })
        )
    }
}
