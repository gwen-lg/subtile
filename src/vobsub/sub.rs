//! # Subtitle data parsing.
//!
//! For background, see [this documentation on the DVD subtitle format][subs].
//!
//! [subs]: http://sam.zoy.org/writings/dvd/subtitles/

use image::{ImageBuffer, Rgba, RgbaImage};
use log::{trace, warn};
use nom::{
    bits::{bits, complete::take as take_bits},
    branch::alt,
    bytes,
    bytes::complete::{tag as tag_bytes, take_until},
    combinator::{map, value},
    multi::{count, many_till},
    number::complete::be_u16,
    sequence::{preceded, Tuple},
    IResult,
};
use std::{cmp::Ordering, fmt};
use thiserror::Error;

use super::{img::decompress, mpeg2::ps, Palette, VobSubError};
use crate::{
    content::{Area, AreaValues},
    util::BytesFormatter,
    vobsub::IResultExt,
};

/// The default time between two adjacent subtitles if no end time is
/// provided.  This is chosen to be a value that's usually representable in
/// `SRT` format, barring rounding errors.
const DEFAULT_SUBTITLE_SPACING: f64 = 0.001;

/// The default length of a subtitle if no end time is provided and no
/// subtitle follows immediately after.
const DEFAULT_SUBTITLE_LENGTH: f64 = 5.0;

/// Parse four 4-bit palette entries.
fn palette_entries(input: &[u8]) -> IResult<&[u8], [u8; 4]> {
    let (input, vec) = bits(count(
        take_bits::<_, _, _, nom::error::Error<(&[u8], usize)>>(4usize),
        4,
    ))(input)?;

    let mut result = [0; 4];
    <[u8; 4] as AsMut<_>>::as_mut(&mut result).clone_from_slice(&vec[0..4]);
    Ok((input, result))
}

/// Parse a 12-bit coordinate value.
fn coordinate(input: (&[u8], usize)) -> IResult<(&[u8], usize), u16> {
    take_bits::<_, _, _, _>(12u8)(input)
}

/// Parse four 12-bit coordinate values as a rectangle (with right and
/// bottom coordinates inclusive).
fn area(input: &[u8]) -> IResult<&[u8], AreaValues> {
    bits(|input| {
        let (input, (x1, x2, y1, y2)) =
            (coordinate, coordinate, coordinate, coordinate).parse(input)?;
        Ok((input, AreaValues { x1, y1, x2, y2 }))
    })(input)
}

/// Parse a pair of 16-bit RLE offsets.
fn rle_offsets(input: &[u8]) -> IResult<&[u8], [u16; 2]> {
    let (input, vec) = bits(count(
        take_bits::<_, _, _, nom::error::Error<(&[u8], usize)>>(16u16),
        2,
    ))(input)?;
    Ok((input, [vec[0], vec[1]]))
}

/// Individual commands which may appear in a control sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ControlCommand<'a> {
    /// Should this subtitle be displayed even if subtitles are turned off?
    Force,
    /// We should start displaying the subtitle at the `date` for this
    /// `ControlSequence`.
    StartDate,
    /// We should stop displaying the subtitle at the `date` for this
    /// `ControlSequence`.
    StopDate,
    /// Map each of the 4 colors in this subtitle to a 4-bit palette.
    Palette([u8; 4]),
    /// Map each of the 4 colors in this subtitle to 4 bits of alpha
    /// channel data.
    Alpha([u8; 4]),
    /// Coordinates at which to display the subtitle.
    Coordinates(AreaValues),
    /// Offsets of first and second scan line in our data buffer.  Note
    /// that the data buffer stores alternating scan lines separately, so
    /// these are the first line in each of the two chunks.
    RleOffsets([u16; 2]),
    /// Unsupported trailing data that we don't know how to parse.
    Unsupported(&'a [u8]),
}

/// Parse a single command in a control sequence.
fn control_command(input: &[u8]) -> IResult<&[u8], ControlCommand> {
    alt((
        value(ControlCommand::Force, tag_bytes(&[0x00])),
        value(ControlCommand::StartDate, tag_bytes(&[0x01])),
        value(ControlCommand::StopDate, tag_bytes(&[0x02])),
        map(
            preceded(tag_bytes(&[0x03]), palette_entries),
            ControlCommand::Palette,
        ),
        map(
            preceded(tag_bytes(&[0x04]), palette_entries),
            ControlCommand::Alpha,
        ),
        map(
            preceded(tag_bytes(&[0x05]), area),
            ControlCommand::Coordinates,
        ),
        map(
            preceded(tag_bytes(&[0x06]), rle_offsets),
            ControlCommand::RleOffsets,
        ),
        // We only capture this so we have something to log.  Note that we
        // may not find the _true_ `ControlCommand::End` in this case, but
        // that doesn't matter, because we'll use the `next` field of
        // `ControlSequence` to find the next `ControlSequence`.
        map(take_until(&[0xff][..]), ControlCommand::Unsupported),
    ))(input)
}

/// The end of a control sequence.
fn control_command_end(input: &[u8]) -> IResult<&[u8], &[u8]> {
    bytes::complete::tag(&[0xff])(input)
}

/// The control packet for a subtitle.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ControlSequence<'a> {
    /// The time associated with this control sequence, specified in
    /// 1/100th of a second after the Presentation Time Stamp for this
    /// subtitle's packet.
    date: u16,
    /// The offset of the next control sequence, relative to ???.  If this
    /// equals the offset of the current control sequence, this is the last
    /// control packet.
    next: u16,
    /// Individual commands in this sequence.
    commands: Vec<ControlCommand<'a>>,
}

/// Parse a single control sequence.
fn control_sequence(input: &[u8]) -> IResult<&[u8], ControlSequence> {
    let (input, (date, next, commands)) = (
        be_u16,
        be_u16,
        many_till(control_command, control_command_end),
    )
        .parse(input)?;
    Ok((
        input,
        ControlSequence {
            date,
            next,
            commands: commands.0,
        },
    ))
}

/// A single subtitle.
#[derive(Clone, PartialEq)]
pub struct Subtitle {
    /// Start time of subtitle, in seconds.
    start_time: f64,
    /// End time of subtitle, in seconds.  This may be missing from certain
    /// subtitles.
    end_time: Option<f64>,
    /// Should this subtitle be shown even when subtitles are off?
    force: bool,
    /// Area coordinates at which to display the subtitle.
    area: Area,
    /// Map each of the 4 colors in this subtitle to a 4-bit palette.
    palette: [u8; 4],
    /// Map each of the 4 colors in this subtitle to 4 bits of alpha
    /// channel data.
    alpha: [u8; 4],
    /// Our decompressed image, stored with 2 bits per byte in row-major
    /// order, that can be used as indices into `palette` and `alpha`.
    raw_image: Vec<u8>,
}

impl Subtitle {
    /// Start time of subtitle, in seconds.
    #[must_use]
    pub const fn start_time(&self) -> f64 {
        self.start_time
    }

    /// End time of subtitle, in seconds. This may be missing from certain
    /// subtitles.
    /// # Panics
    /// Will panic if `end_time` is not set. As it should be set before returning subtitle.
    /// If happened `end_time` is called to soon, or a change has broken the intended operation.
    #[must_use]
    pub fn end_time(&self) -> f64 {
        self.end_time
            .expect("end time should have been set before returning subtitle")
    }

    /// Should this subtitle be shown even when subtitles are off?
    #[must_use]
    pub const fn force(&self) -> bool {
        self.force
    }

    /// Coordinates at which to display the subtitle.
    #[must_use]
    pub const fn area(&self) -> &Area {
        &self.area
    }

    /// Map each of the 4 colors in this subtitle to a 4-bit palette.
    #[must_use]
    pub const fn palette(&self) -> &[u8; 4] {
        &self.palette
    }

    /// Map each of the 4 colors in this subtitle to 4 bits of alpha
    /// channel data.
    #[must_use]
    pub const fn alpha(&self) -> &[u8; 4] {
        &self.alpha
    }

    /// Our decompressed image, stored with 2 bits per byte in row-major
    /// order, that can be used as indices into `palette` and `alpha`.
    #[must_use]
    pub fn raw_image(&self) -> &[u8] {
        &self.raw_image
    }

    /// Decompress to subtitle to an RBGA image.
    #[must_use]
    pub fn to_image(&self, palette: &Palette) -> RgbaImage {
        let width = u32::from(self.area.width());
        let height = u32::from(self.area.height());
        ImageBuffer::from_fn(width, height, |x, y| {
            let offset = cast::usize(y * width + x);
            // We need to subtract the raw index from 3 to get the same
            // results as everybody else.  I found this by inspecting the
            // Handbrake subtitle decoding routines.
            let px = usize::from(3 - self.raw_image[offset]);
            let rgb = palette[usize::from(self.palette[px])].0;
            let a = self.alpha[px];
            let aa = a << 4 | a;
            Rgba([rgb[0], rgb[1], rgb[2], aa])
        })
    }
}

impl fmt::Debug for Subtitle {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Subtitle")
            .field("start_time", &self.start_time)
            .field("end_time", &self.end_time)
            .field("force", &self.force)
            .field("area", &self.area)
            .field("palette", &self.palette)
            .field("alpha", &self.alpha)
            .finish_non_exhaustive()
    }
}

/// Parse a single `u16` value from a buffer.  We don't use `nom` for this
/// because it has an inconvenient error type.
fn parse_be_u16_as_usize(buff: &[u8]) -> Result<(&[u8], usize), VobSubError> {
    if buff.len() < 2 {
        Err(VobSubError::BufferTooSmallForU16)
    } else {
        Ok((&buff[2..], usize::from(buff[0]) << 8 | usize::from(buff[1])))
    }
}

/// Errors for missing subtitle part after parsing.
#[derive(Debug, Error)]
pub enum ErrorMissing {
    /// No start time.
    #[error("no start time")]
    StartTime,

    /// No area coordinates
    #[error("no area coordinates")]
    Area,

    /// No palette
    #[error("no palette")]
    Palette,

    /// No alpha palette
    #[error("no alpha palette")]
    AlphaPalette,

    /// No RLE offsets
    #[error("no RLE offsets")]
    RleOffset,
}

/// Parse a subtitle.
fn subtitle(raw_data: &[u8], base_time: f64) -> Result<Subtitle, VobSubError> {
    // This parser is somewhat non-standard, because we need to work with
    // explicit offsets into `packet` in several places.

    // Figure out where our control data starts.
    if raw_data.len() < 2 {
        return Err(VobSubError::UnexpectedEndOfSubtitleData);
    }
    let (_, initial_control_offset) = parse_be_u16_as_usize(&raw_data[2..])?;

    // Declare data we want to collect from our control packets.
    let mut start_time = None;
    let mut end_time = None;
    let mut force = false;
    let mut area = None;
    let mut palette = None;
    let mut alpha = None;
    let mut rle_offsets = None;

    // Loop over the individual control sequences.
    let mut control_offset = initial_control_offset;
    loop {
        trace!("looking for control sequence at: 0x{:x}", control_offset);
        if control_offset >= raw_data.len() {
            return Err(VobSubError::ControlOffsetBiggerThanPacket {
                offset: control_offset,
                packet: raw_data.len(),
            });
        }

        let control_data = &raw_data[control_offset..];
        let (_, control) = control_sequence(control_data)
            .to_result()
            .map_err(VobSubError::ControlSequence)?;

        trace!("parsed control sequence: {:?}", &control);

        // Extract as much data as we can from this control sequence.
        let time = base_time + f64::from(control.date) / 100.0;
        for command in control.commands {
            match command {
                ControlCommand::Force => {
                    force = true;
                }
                ControlCommand::StartDate => {
                    start_time = start_time.or(Some(time));
                }
                ControlCommand::StopDate => {
                    end_time = end_time.or(Some(time));
                }
                ControlCommand::Palette(p) => {
                    palette = palette.or(Some(p));
                }
                ControlCommand::Alpha(a) => {
                    alpha = alpha.or(Some(a));
                }
                ControlCommand::Coordinates(c) => {
                    let cmd_area = Area::try_from(c)?;
                    area = area.or(Some(cmd_area));
                }
                ControlCommand::RleOffsets(r) => {
                    rle_offsets = Some(r);
                }
                ControlCommand::Unsupported(b) => {
                    warn!("unsupported control sequence: {:?}", BytesFormatter(b));
                }
            }
        }

        // Figure out where to look for the next control sequence,
        // if any.
        let next_control_offset = usize::from(control.next);
        match control_offset.cmp(&next_control_offset) {
            Ordering::Greater => {
                return Err(VobSubError::ControlOffsetWentBackwards);
            }
            Ordering::Equal => {
                // This points back at us, so we're the last packet.
                break;
            }
            Ordering::Less => {
                control_offset = next_control_offset;
            }
        }
    }

    // Make sure we found all the control commands that we expect.
    let start_time = start_time.ok_or(ErrorMissing::StartTime)?;
    let area = area.ok_or(ErrorMissing::Area)?;
    let palette = palette.ok_or(ErrorMissing::Palette)?;
    let alpha = alpha.ok_or(ErrorMissing::AlphaPalette)?;
    let rle_offsets = rle_offsets.ok_or(ErrorMissing::RleOffset)?;

    // Decompress our image.
    //
    // We know the starting points of each set of scan lines, but we don't
    // really know where they end, because encoders like to reuse bytes
    // that they're already using for something else.  For example, the
    // last few bytes of the first set of scan lines may overlap with the
    // first bytes of the second set of scanlines, and the last bytes of
    // the second set of scan lines may overlap with the start of the
    // control sequence.  For now, we limit it to the first two bytes of
    // the control packet, which are usually `[0x00, 0x00]`.  (We might
    // actually want to remove `end` entirely here and allow the scan lines
    // to go to the end of the packet, but I've never seen that in
    // practice.)
    let start_0 = usize::from(rle_offsets[0]);
    let start_1 = usize::from(rle_offsets[1]);
    let end = initial_control_offset + 2;
    if start_0 > start_1 || start_1 > end {
        return Err(VobSubError::InvalidScanLineOffsets {
            start_0,
            start_1,
            end,
        });
    }
    let image = decompress(
        area.size(),
        [&raw_data[start_0..end], &raw_data[start_1..end]],
    )?;

    // Return our parsed subtitle.
    let result = Subtitle {
        start_time,
        end_time,
        force,
        area,
        palette,
        alpha,
        raw_image: image,
    };
    trace!("Parsed subtitle: {:?}", &result);
    Ok(result)
}

/// Like `?` and `try!`, but assume that we're working with
/// `Option<Result<T, E>>` instead of `Result<T, E>`, and pass through
/// `None`.
macro_rules! try_iter {
    ($e:expr) => {
        match $e {
            None => return None,
            Some(Err(e)) => return Some(Err(From::from(e))),
            Some(Ok(value)) => value,
        }
    };
}

/// An internal iterator over subtitles.  These subtitles may not have a
/// valid `end_time`, so we'll try to fix them up before letting the user
/// see them.
struct SubtitlesInternal<'a> {
    pes_packets: ps::PesPackets<'a>,
}

impl<'a> Iterator for SubtitlesInternal<'a> {
    type Item = Result<Subtitle, VobSubError>;

    fn next(&mut self) -> Option<Self::Item> {
        profiling::scope!("SubtitlesInternal next");

        // Get the PES packet containing the first chunk of our subtitle.
        let first: ps::PesPacket = try_iter!(self.pes_packets.next());

        // Fetch useful information from our first packet.
        let pts_dts = match first.pes_packet.header_data.pts_dts {
            Some(v) => v,
            None => return Some(Err(VobSubError::MissingTimingForSubtitle)),
        };
        let base_time = pts_dts.pts.as_seconds();
        let substream_id = first.pes_packet.substream_id;

        // Figure out how many total bytes we'll need to collect from one
        // or more PES packets, and collect the first chunk into a buffer.
        if first.pes_packet.data.len() < 2 {
            return Some(Err(VobSubError::PacketTooShort));
        }
        let wanted =
            usize::from(first.pes_packet.data[0]) << 8 | usize::from(first.pes_packet.data[1]);
        let mut sub_packet = Vec::with_capacity(wanted);
        sub_packet.extend_from_slice(first.pes_packet.data);

        // Keep fetching more packets until we have enough.
        while sub_packet.len() < wanted {
            // Get the next PES packet in the Program Stream.
            let next: ps::PesPacket = try_iter!(self.pes_packets.next());

            // Make sure this is part of the same subtitle stream.  This is
            // mostly just paranoia; I don't expect this to happen.
            if next.pes_packet.substream_id != substream_id {
                warn!(
                    "Found subtitle for stream 0x{:x} while looking for 0x{:x}",
                    next.pes_packet.substream_id, substream_id
                );
                continue;
            }

            // Add the extra bytes to our buffer.
            sub_packet.extend_from_slice(next.pes_packet.data);
        }

        // Check to make sure we didn't get too _many_ bytes.  Again, this
        // is paranoia.
        if sub_packet.len() > wanted {
            warn!(
                "Found 0x{:x} bytes of data in subtitle packet, wanted 0x{:x}",
                sub_packet.len(),
                wanted
            );
            sub_packet.truncate(wanted);
        }

        // Parse our subtitle buffer.
        Some(subtitle(&sub_packet, base_time))
    }
}

/// An iterator over subtitles.
pub struct Subtitles<'a> {
    internal: SubtitlesInternal<'a>,
    prev: Option<Subtitle>,
}

impl<'a> Iterator for Subtitles<'a> {
    type Item = Result<Subtitle, VobSubError>;

    // This whole routine exists to make sure that `end_time` is set to a
    // useful value even if the subtitles themselves didn't supply one.
    // I'm not even sure this is valid, but it has been observed in the
    // wild.
    fn next(&mut self) -> Option<Self::Item> {
        profiling::scope!("Subtitles next");

        // If we don't currently have a previous subtitle, attempt to fetch
        // one.
        if self.prev.is_none() {
            match self.internal.next() {
                Some(Ok(sub)) => {
                    self.prev = Some(sub);
                }
                other => return other,
            }
        }
        debug_assert!(self.prev.is_some());

        match self.internal.next() {
            // We have a another subtitle!  We want to return `self.prev`
            // and store the new subtitle as `self.prev`.
            Some(Ok(curr)) => {
                // `unwrap` is safe because of the invariant above.
                let mut prev = self.prev.take().unwrap();
                if prev.end_time.is_none() {
                    // Our subtitle has no end time, so end it just before
                    // the next subtitle.
                    let new_end = curr.start_time - DEFAULT_SUBTITLE_SPACING;
                    let alt_end = prev.start_time + DEFAULT_SUBTITLE_LENGTH;
                    prev.end_time = Some(new_end.min(alt_end));
                }
                self.prev = Some(curr);
                Some(Ok(prev))
            }
            // We encountered an error.  We could, I suppose, attempt to
            // first return `self.prev` and save the error for next time,
            // but that's too much trouble.
            Some(Err(err)) => Some(Err(err)),
            // The only subtitle left to return is `self.prev`.
            None => {
                self.prev.take().map(|mut sub| {
                    if sub.end_time.is_none() {
                        // Our subtitle has no end time, and it's the last
                        // subtitle, so just pick something.
                        sub.end_time = Some(sub.start_time + DEFAULT_SUBTITLE_LENGTH);
                    }
                    Ok(sub)
                })
            }
        }
    }
}

/// Return an iterator over the subtitles in this data stream.
#[must_use]
pub const fn subtitles(input: &[u8]) -> Subtitles {
    Subtitles {
        internal: SubtitlesInternal {
            pes_packets: ps::pes_packets(input),
        },
        prev: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vobsub::idx;

    #[test]
    fn parse_palette_entries() {
        assert_eq!(
            palette_entries(&[0x03, 0x10][..]),
            IResult::Ok((&[][..], [0x00, 0x03, 0x01, 0x00]))
        );
    }

    #[test]
    fn parse_control_sequence() {
        let input_1 = &[
            0x00, 0x00, 0x0f, 0x41, 0x01, 0x03, 0x03, 0x10, 0x04, 0xff, 0xf0, 0x05, 0x29, 0xb4,
            0xe6, 0x3c, 0x54, 0x00, 0x06, 0x00, 0x04, 0x07, 0x7b, 0xff,
        ][..];
        let expected_1 = ControlSequence {
            date: 0x0000,
            next: 0x0f41,
            commands: vec![
                ControlCommand::StartDate,
                ControlCommand::Palette([0x0, 0x3, 0x1, 0x0]),
                ControlCommand::Alpha([0xf, 0xf, 0xf, 0x0]),
                ControlCommand::Coordinates(AreaValues {
                    x1: 0x29b,
                    x2: 0x4e6,
                    y1: 0x3c5,
                    y2: 0x400,
                }),
                ControlCommand::RleOffsets([0x0004, 0x077b]),
            ],
        };
        assert_eq!(
            control_sequence(input_1),
            IResult::Ok((&[][..], expected_1))
        );

        let input_2 = &[0x00, 0x77, 0x0f, 0x41, 0x02, 0xff][..];
        let expected_2 = ControlSequence {
            date: 0x0077,
            next: 0x0f41,
            commands: vec![ControlCommand::StopDate],
        };
        assert_eq!(
            control_sequence(input_2),
            IResult::Ok((&[][..], expected_2))
        );

        // An out of order example.
        let input_3 = &[
            0x00, 0x00, 0x0b, 0x30, 0x01, 0x00, // ...other commands would appear here...
            0xff,
        ][..];
        let expected_3 = ControlSequence {
            date: 0x0000,
            next: 0x0b30,
            commands: vec![ControlCommand::StartDate, ControlCommand::Force],
        };
        assert_eq!(
            control_sequence(input_3),
            IResult::Ok((&[][..], expected_3))
        );
    }

    #[test]
    fn parse_subtitles() {
        //use env_logger;
        use std::fs;
        use std::io::prelude::*;

        //let _ = env_logger::init();

        let mut f = fs::File::open("./fixtures/example.sub").unwrap();
        let mut buffer = vec![];
        f.read_to_end(&mut buffer).unwrap();
        let mut subs = subtitles(&buffer);
        let sub1 = subs.next().expect("missing sub 1").unwrap();
        assert!(sub1.start_time - 49.4 < 0.1);
        assert!(sub1.end_time.unwrap() - 50.9 < 0.1);
        assert!(!sub1.force);
        assert_eq!(
            sub1.area,
            Area::try_from(AreaValues {
                x1: 750,
                y1: 916,
                x2: 1172,
                y2: 966
            })
            .unwrap()
        );
        assert_eq!(sub1.palette, [0, 3, 1, 0]);
        assert_eq!(sub1.alpha, [15, 15, 15, 0]);
        subs.next().expect("missing sub 2").unwrap();
        assert!(subs.next().is_none());
    }

    #[test]
    fn parse_subtitles_from_subtitle_edit() {
        //use env_logger;
        use idx::Index;
        //let _ = env_logger::init();
        let idx = Index::open("./fixtures/tiny.idx").unwrap();
        let mut subs = idx.subtitles();
        subs.next().expect("missing sub").unwrap();
        assert!(subs.next().is_none());
    }

    #[test]
    fn parse_fuzz_corpus_seeds() {
        //use env_logger;
        use idx::Index;
        //let _ = env_logger::init();

        // Make sure these two fuzz corpus inputs still work, and that they
        // return the same subtitle data.
        let tiny = Index::open("./fixtures/tiny.idx")
            .unwrap()
            .subtitles()
            .next()
            .unwrap()
            .unwrap();
        let split = Index::open("./fixtures/tiny-split.idx")
            .unwrap()
            .subtitles()
            .next()
            .unwrap()
            .unwrap();
        assert_eq!(tiny, split);
    }
}
