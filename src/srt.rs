//! SubRip/Srt functionality
use std::io;

use crate::time::TimeSpan;

#[repr(transparent)]
pub struct TimePointSrt(TimePoint);

impl From<TimePoint> for TimePointSrt {
    fn from(value: TimePoint) -> Self {
        Self(value)
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
        let start = time_span.start;
        let end = time_span.end;
        writeln!(writer, "{line_num}\n{start} --> {end}\n{text}")
    }
}
