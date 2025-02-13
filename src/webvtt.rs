//! `WebVTT` functionality
use std::fmt;

use crate::time::TimePoint;

/// Extend `TimePoint` for implement `WebVTT` specific `Display`.
#[repr(transparent)]
pub struct TimePointVtt(TimePoint);

impl From<TimePoint> for TimePointVtt {
    fn from(value: TimePoint) -> Self {
        Self(value)
    }
}

impl fmt::Display for TimePointVtt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_separator(f, '.')
    }
}
