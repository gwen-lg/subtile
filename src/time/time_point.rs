use core::fmt;
use std::ops::Neg;

/// Define a time in milliseconds
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimePoint(i64);

impl TimePoint {
    /// Create a `TimePoint` from miliseconds
    #[must_use]
    pub const fn from_msecs(time: i64) -> Self {
        Self(time)
    }

    /// Create a `TimePoint` from seconds
    ///
    /// # Panics
    ///
    /// Will panics if the `seconds` value fill as parameter is to big to be store as
    /// millisecond in a i64.
    #[must_use]
    pub fn from_secs(seconds: f64) -> Self {
        let msecs = cast::i64(seconds * 1000.0).unwrap();
        Self(msecs)
    }

    /// Convert to seconds
    #[must_use]
    pub fn to_secs(self) -> f64 {
        self.0 as f64 / 1000.
    }

    const fn msecs(self) -> i64 {
        self.0
    }

    const fn secs(self) -> i64 {
        self.0 / 1000
    }

    const fn mins(self) -> i64 {
        self.0 / (60 * 1000)
    }

    const fn hours(self) -> i64 {
        self.0 / (60 * 60 * 1000)
    }
    const fn mins_comp(self) -> i64 {
        self.mins() % 60
    }

    const fn secs_comp(self) -> i64 {
        self.secs() % 60
    }

    const fn msecs_comp(self) -> i64 {
        self.msecs() % 1000
    }
}

impl Neg for TimePoint {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_point_creation() {
        assert_eq!(TimePoint::from_msecs(6751), TimePoint(6751));
        assert_eq!(TimePoint::from_msecs(142), TimePoint::from_secs(0.142));
    }

    #[test]
    fn time_point_creation_with_too_mutch_decimals() {
        assert_eq!(TimePoint::from_msecs(265), TimePoint::from_secs(0.265_579));
        assert_eq!(TimePoint(142), TimePoint::from_secs(0.142_75));
    }

    #[test]
    fn time_point_msecs() {
        const TIME: i64 = 62487;
        assert_eq!(TimePoint::from_msecs(TIME).msecs(), TIME);
    }

    #[test]
    fn time_point_secs() {
        const TIME: f64 = 624.87;
        assert_eq!(TimePoint::from_secs(TIME).secs(), 624);
    }
}
