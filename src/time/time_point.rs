use core::fmt;
use std::ops::Neg;

/// Define a time in milliseconds
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimePoint(i64);

impl TimePoint {
    /// Create a `TimePoint` from miliseconds
    #[must_use]
    pub fn from_msecs(time: i64) -> Self {
        Self(time)
    }

    /// Convert to seconds
    #[must_use]
    pub fn to_secs(self) -> f64 {
        self.0 as f64 / 1000.
    }

    fn msecs(self) -> i64 {
        self.0
    }

    fn secs(self) -> i64 {
        self.0 / 1000
    }

    fn mins(self) -> i64 {
        self.0 / (60 * 1000)
    }

    fn hours(self) -> i64 {
        self.0 / (60 * 60 * 1000)
    }
    fn mins_comp(self) -> i64 {
        self.mins() % 60
    }

    fn secs_comp(self) -> i64 {
        self.secs() % 60
    }

    fn msecs_comp(self) -> i64 {
        self.msecs() % 1000
    }
}

impl Neg for TimePoint {
    type Output = TimePoint;
    fn neg(self) -> TimePoint {
        TimePoint(-self.0)
    }
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = if self.0 < 0 { -*self } else { *self };
        write!(
            f,
            "{}{:02}:{:02}:{:02},{:03}",
            if self.0 < 0 { "-" } else { "" },
            t.hours(),
            t.mins_comp(),
            t.secs_comp(),
            t.msecs_comp()
        )
    }
}
//TODO: add tests
