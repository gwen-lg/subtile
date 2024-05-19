use super::TimePoint;
use core::fmt::{self, Debug};

/// Define a time span with a start time and an end time.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TimeSpan {
    /// Start time of the span
    pub start: TimePoint,
    /// End time of the span
    pub end: TimePoint,
}

impl TimeSpan {
    /// Create a new `TimeSpan` from a start and an end.
    #[must_use]
    pub const fn new(start: TimePoint, end: TimePoint) -> Self {
        Self { start, end }
    }
}

impl Debug for TimeSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} --> {}", self.start, self.end)
    }
}
