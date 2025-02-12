use super::TimePoint;
use core::fmt::{self, Debug};

/// Define a time span with a start time and an end time.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
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
        write!(f, "{:?} --> {:?}", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_span_creation() {
        assert_eq!(
            TimeSpan::new(TimePoint::from_msecs(0), TimePoint::from_secs(1.34)),
            TimeSpan {
                start: TimePoint::from_msecs(0),
                end: TimePoint::from_secs(1.34)
            }
        );
    }

    #[test]
    fn time_span_equality() {
        let time_span_0_1 = TimeSpan::new(TimePoint::from_msecs(0), TimePoint::from_secs(1.34));
        let time_span_1_2 = TimeSpan::new(TimePoint::from_msecs(1245), TimePoint::from_secs(2.34));
        assert_eq!(
            time_span_0_1,
            TimeSpan::new(TimePoint::from_msecs(0), TimePoint::from_secs(1.34))
        );
        assert_eq!(
            time_span_1_2,
            TimeSpan {
                start: TimePoint::from_msecs(1245),
                end: TimePoint::from_secs(2.34)
            }
        );
    }

    #[test]
    fn time_span_nequality() {
        let time_span_0_1 = TimeSpan::new(TimePoint::from_msecs(0), TimePoint::from_secs(1.34));
        let time_span_0_2 = TimeSpan::new(TimePoint::from_msecs(0), TimePoint::from_secs(2.34));
        let time_span_1_2 = TimeSpan::new(TimePoint::from_msecs(1245), TimePoint::from_secs(2.34));
        assert_ne!(time_span_0_1, time_span_0_2);
        assert_ne!(time_span_0_2, time_span_1_2);
    }
}
