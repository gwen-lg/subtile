//! Run-length encoded image format for subtitles.

use log::trace;
use nom::{
    bits::complete::{tag as tag_bits, take as take_bits},
    branch::alt,
    combinator::value,
    sequence::{preceded, Tuple},
    IResult,
};
use thiserror::Error;

use super::IResultExt;
use crate::{content::Size, util::BytesFormatter};

use super::NomError;

/// Errors of `vobsub` img management.
#[derive(Error, Debug)]
pub enum Error {
    /// If there is more data to write than the space in output.
    #[error("output parameter is too small (size:{output_size}) for write scanline data (size:{data_size})")]
    ToSmallOutput {
        data_size: usize,
        output_size: usize,
    },

    /// If index value is bigger than the image width.
    #[error("Scan line is longer than image width: [{x},{width}]")]
    ScanLineLongerThanWidth { x: usize, width: usize },

    /// Forward scan line parsind error.
    #[error("Parsing scan line failed")]
    ScanLineParsing(#[source] NomError),
}

/// A run-length encoded value.
#[derive(Debug)]
struct Rle {
    /// The number of times to repeat this value.  A value of 0 indicates that
    /// we should fill to the end of the line.
    cnt: u16,
    /// The value to repeat.  This is 2 bits wide.
    val: u8,
}

/// Parse the count for a `Rle`.
fn count(input: (&[u8], usize)) -> IResult<(&[u8], usize), u16> {
    // Fill to end of line.
    let end_of_line = value(0, tag_bits(0, 14u16));
    // Count for 4-nibble RLE.
    let count4 = preceded(tag_bits(0, 6u8), take_bits(8u16));
    // Count for 3-nibble RLE.
    let count3 = preceded(tag_bits(0, 4u8), take_bits(6u16));
    // Count for 2-nibble RLE.
    let count2 = preceded(tag_bits(0, 2u8), take_bits(4u16));
    // Count for 1-nibble RLE.
    let count1 = take_bits(2u16);
    alt((end_of_line, count4, count3, count2, count1))(input)
}

/// Parse an `Rle`.
fn rle(input: (&[u8], usize)) -> IResult<(&[u8], usize), Rle> {
    let take_val = take_bits(2u8);
    let (input, (cnt, val)) = (count, take_val).parse(input)?;
    Ok((input, Rle { cnt, val }))
}

/// Decompress the scan-line `input` into `output`, returning the number of
/// input bytes consumed.
fn scan_line(input: &[u8], output: &mut [u8]) -> Result<usize, Error> {
    trace!("scan line starting with {:?}", BytesFormatter(input));
    let width = output.len();
    let mut x = 0;
    let mut pos = (input, 0);
    while x < width {
        let (new_pos, run) = rle(pos).to_result().map_err(Error::ScanLineParsing)?;

        //trace!("RLE: {:?}", &run);
        pos = new_pos;
        let count = if run.cnt == 0 {
            width - x
        } else {
            usize::from(run.cnt)
        };
        if x + count > output.len() {
            return Err(Error::ToSmallOutput {
                data_size: x + count,
                output_size: output.len(),
            });
        }
        output[x..x + count].fill(run.val);
        x += count;
    }
    if x > width {
        return Err(Error::ScanLineLongerThanWidth { x, width });
    }
    // Round up to the next full byte.
    if pos.1 > 0 {
        pos = (&pos.0[1..], 0);
    }
    Ok(input.len() - pos.0.len())
}

/// Decompress a run-length encoded image, and return a vector in row-major
/// order, starting at the upper-left and scanning right and down, with one
/// byte for each 2-bit value.
#[profiling::function]
pub fn decompress(size: Size, data: [&[u8]; 2]) -> Result<Vec<u8>, Error> {
    trace!(
        "decompressing image {:?}, max: [0x{:x}, 0x{:x}]",
        &size,
        data[0].len(),
        data[1].len()
    );
    let mut img = vec![0; size.w * size.h];
    let mut offsets = [0; 2];
    for y in 0..size.h {
        let odd = y % 2;
        trace!("line {:?}, offset 0x{:x}", y, offsets[odd]);
        let consumed = scan_line(
            &data[odd][offsets[odd]..],
            &mut img[y * size.w..(y + 1) * size.w],
        )?;
        offsets[odd] += consumed;
    }
    // TODO: Warn if we didn't consume everything.
    Ok(img)
}
