/// SubRip/Srt fonctionnality
use std::io;

use crate::time::TimeSpan;

/// Write subtitles in srt format
pub fn write_srt(
    data: &[(TimeSpan, String)],
    writer: &mut impl io::Write,
) -> Result<(), io::Error> {
    data.iter()
        .enumerate()
        .try_for_each(write_srt_line(writer))?;

    Ok(())
}

/// Write an subtitle line in Srt format
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
