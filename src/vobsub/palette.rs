use failure::format_err;
use image::Rgb;

/// Parse a single hexadecimal digit.
named!(
    hex_digit<u8>,
    map!(one_of!(b"0123456789abcdefABCDEF"), |c: char| -> u8 {
        cast::u8(c.to_digit(16).unwrap()).unwrap()
    })
);

/// Parse a single byte hexadecimal byte.
named!(
    hex_u8<u8>,
    do_parse!(
        h1: call!(hex_digit) >>
        h2: call!(hex_digit) >>
        (h1 << 4 | h2)
    )
);

/// Parse a 3-byte hexadecimal RGB color.
named!(
    rgb<Rgb<u8>>,
    map!(count_fixed!(u8, call!(hex_u8), 3), |rgb| { Rgb(rgb) })
);

/// The 16-color pallette used by the subtitles.
pub type Palette = [Rgb<u8>; 16];

named!(
    pub palette<Palette>,
    map_res!(separated_list!(tag!(b", "), call!(rgb)), |vec: Vec<
        Rgb<u8>,
    >| {
        if vec.len() != 16 {
            return Err(format_err!("Palettes must have 16 entries"));
        }
        // Coerce vector to known-size slice.  Based on
        // http://stackoverflow.com/q/25428920/12089.
        let mut result = [Rgb([0, 0, 0]); 16];
        <[Rgb<u8>; 16] as AsMut<_>>::as_mut(&mut result).clone_from_slice(&vec[0..16]);
        Ok(result)
    })
);

#[cfg(test)]
mod tests {
    use super::*;

    use image::Rgb;

    #[test]
    fn parse_rgb() {
        use nom::IResult;
        assert_eq!(
            rgb(&b"1234ab"[..]),
            IResult::Done(&b""[..], Rgb::<u8>([0x12, 0x34, 0xab]))
        );
    }

    #[test]
    fn parse_palette() {
        use nom::IResult;
        let input = b"\
000000, f0f0f0, cccccc, 999999, 3333fa, 1111bb, fa3333, bb1111, \
33fa33, 11bb11, fafa33, bbbb11, fa33fa, bb11bb, 33fafa, 11bbbb";
        assert_eq!(
            palette(input),
            IResult::Done(
                &[][..],
                [
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
                    Rgb([0x11, 0xbb, 0xbb])
                ]
            )
        );
    }
}
