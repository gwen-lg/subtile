use super::TimePoint;

/// Define a time span with a start time and an end time.
#[derive(Clone, Copy)]
pub struct TimeSpan {
    /// Start time of the span
    pub start: TimePoint,
    /// End time of the span
    pub end: TimePoint,
}

impl TimeSpan {
    /// Create a new `TimeSpan` from a start and an end.
    #[must_use]
    pub fn new(start: TimePoint, end: TimePoint) -> Self {
        Self { start, end }
    }
}
