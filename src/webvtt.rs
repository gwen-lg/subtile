//! `WebVTT` functionality
use std::{fmt, io};

use crate::time::{TimePoint, TimeSpan};

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

/// Write a subtitles line in `vtt` format
/// # Errors
///
/// Will return `Err` if writing in `writer` return an `Err`.
pub fn write_line(
    writer: &mut impl io::Write,
    time: &TimeSpan,
    text: &str,
) -> Result<(), io::Error> {
    let start = TimePointVtt(time.start);
    let end = TimePointVtt(time.end);
    writeln!(writer, "{start} --> {end}\n{text}\n")
}
