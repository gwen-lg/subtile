use image::{Luma, Pixel, Rgb};
use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    multi::separated_list0,
    sequence::tuple,
    IResult,
};

use super::VobSubError;

pub const DEFAULT_PALETTE: Palette = [
    Rgb([0x00, 0x00, 0x00]),
    Rgb([0xf0, 0xf0, 0xf0]),
    Rgb([0xcc, 0xcc, 0xcc]),
    Rgb([0x99, 0x99, 0x99]),
    Rgb([0x33, 0x33, 0xfa]),
    Rgb([0x11, 0x11, 0xbb]),
    Rgb([0xfa, 0x33, 0x33]),
    Rgb([0xbb, 0x11, 0x11]),
    Rgb([0x33, 0xfa, 0x33]),
    Rgb([0x11, 0xbb, 0x11]),
    Rgb([0xfa, 0xfa, 0x33]),
    Rgb([0xbb, 0xbb, 0x11]),
    Rgb([0xfa, 0x33, 0xfa]),
    Rgb([0xbb, 0x11, 0xbb]),
    Rgb([0x33, 0xfa, 0xfa]),
    Rgb([0x11, 0xbb, 0xbb]),
];

/// Parse a single hexadecimal digit.
fn from_hex(input: &[u8]) -> std::result::Result<u8, std::num::ParseIntError> {
    let input = std::str::from_utf8(input).unwrap();
    u8::from_str_radix(input, 16)
}

/// Parse a single byte hexadecimal byte.
fn hex_primary(input: &[u8]) -> IResult<&[u8], u8> {
    map_res(
        take_while_m_n(2, 2, |c: u8| c.is_ascii_hexdigit()),
        from_hex,
    )(input)
}

/// Parse a 3-byte hexadecimal `RGB` color.
fn hex_rgb(input: &[u8]) -> IResult<&[u8], Rgb<u8>> {
    let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

    Ok((input, Rgb([red, green, blue])))
}

/// The 16-color palette used by the subtitles.
pub type Palette = [Rgb<u8>; 16];

/// Parse a text as Palette
/// # Errors
///
/// Will return `Err` if the input don't have 16 entries.
pub fn palette(input: &[u8]) -> IResult<&[u8], Palette> {
    let res = map_res(separated_list0(tag(b", "), hex_rgb), |vec: Vec<Rgb<u8>>| {
        if vec.len() != 16 {
            return Err(VobSubError::PaletteInvalidEntriesNumbers(vec.len()));
        }
        // Coerce vector to known-size slice.  Based on
        // http://stackoverflow.com/q/25428920/12089.
        let mut result = [Rgb([0, 0, 0]); 16];
        <[Rgb<u8>; 16] as AsMut<_>>::as_mut(&mut result).clone_from_slice(&vec[0..16]);
        Ok(result)
    })(input);
    res
}

/// The 16-luminance palette gene.
pub type PaletteLuma = [Luma<u8>; 16];

/// Convert an sRGB palette to a luminance palette.
#[must_use]
pub fn palette_rgb_to_luminance(palette: &Palette) -> PaletteLuma {
    palette.map(|rgb| rgb.to_luma())
}

#[cfg(test)]
mod tests {
    use super::*;

    use image::Rgb;

    #[test]
    fn parse_rgb() {
        use nom::IResult;
        assert_eq!(
            hex_rgb(&b"1234ab"[..]),
            IResult::Ok((&b""[..], Rgb::<u8>([0x12, 0x34, 0xab])))
        );
    }

    #[test]
    fn parse_palette() {
        use nom::IResult;
        let input = b"\
000000, f0f0f0, cccccc, 999999, 3333fa, 1111bb, fa3333, bb1111, \
33fa33, 11bb11, fafa33, bbbb11, fa33fa, bb11bb, 33fafa, 11bbbb";
        assert_eq!(palette(input), {
            let palette = [
                Rgb([0x00, 0x00, 0x00]),
                Rgb([0xf0, 0xf0, 0xf0]),
                Rgb([0xcc, 0xcc, 0xcc]),
                Rgb([0x99, 0x99, 0x99]),
                Rgb([0x33, 0x33, 0xfa]),
                Rgb([0x11, 0x11, 0xbb]),
                Rgb([0xfa, 0x33, 0x33]),
                Rgb([0xbb, 0x11, 0x11]),
                Rgb([0x33, 0xfa, 0x33]),
                Rgb([0x11, 0xbb, 0x11]),
                Rgb([0xfa, 0xfa, 0x33]),
                Rgb([0xbb, 0xbb, 0x11]),
                Rgb([0xfa, 0x33, 0xfa]),
                Rgb([0xbb, 0x11, 0xbb]),
                Rgb([0x33, 0xfa, 0xfa]),
                Rgb([0x11, 0xbb, 0xbb]),
            ];
            IResult::Ok((&[][..], palette))
        });
    }
}
