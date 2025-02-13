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
        .try_for_each(|(idx, (time_span, text))| {
            let line_num = idx + 1;
            write_line(writer, line_num, time_span, text.as_str())
        })?;

    Ok(())
}

/// Write a subtitle line in `srt` format
/// # Errors
///
/// Will return `Err` if writing in `writer` return an `Err`.
pub fn write_line(
    writer: &mut impl io::Write,
    line_idx: usize,
    time: &TimeSpan,
    text: &str,
) -> Result<(), io::Error> {
    let start = TimePointSrt(time.start);
    let end = TimePointSrt(time.end);
    writeln!(writer, "{line_idx}\n{start} --> {end}\n{text}")
}
