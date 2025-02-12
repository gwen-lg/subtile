//! SubRip/Srt functionality
use std::{fmt, io};

use crate::time::{TimePoint, TimeSpan};

/// Extend `TimePoint` for implement `Srt` specific `Display`.
#[repr(transparent)]
pub struct TimePointSrt(TimePoint);

impl From<TimePoint> for TimePointSrt {
    fn from(value: TimePoint) -> Self {
        Self(value)
    }
}

impl fmt::Display for TimePointSrt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_separator(f, ',')
    }
}

/// Write subtitles in `srt` format
/// # Errors
///
/// Will return `Err` if write in `writer` return an `Err`.
pub fn write_srt(
    writer: &mut impl io::Write,
    subtitles: &[(TimeSpan, String)],
) -> Result<(), io::Error> {
    subtitles
        .iter()
        .enumerate()
        .try_for_each(write_srt_line(writer))?;

    Ok(())
}

/// Write an subtitle line in `srt` format
fn write_srt_line(
    writer: &mut impl io::Write,
) -> impl FnMut((usize, &(TimeSpan, String))) -> Result<(), io::Error> + '_ {
    |(idx, (time_span, text))| {
        let line_num = idx + 1;
        let start = TimePointSrt(time_span.start);
        let end = TimePointSrt(time_span.end);
        writeln!(writer, "{line_num}\n{start} --> {end}\n{text}")
    }
}
