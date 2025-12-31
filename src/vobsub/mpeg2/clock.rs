use nom::{
    bits::complete::{tag, take},
    IResult, Parser as _,
};
use std::fmt;

/// This represents the 90 kHz, 33-bit [System Time Clock][STC] (`STC`) and
/// the 9-bit `STC` extension value, which represents 1/300th of a tick.
///
/// [STC]: http://www.bretl.com/mpeghtml/STC.HTM
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Clock {
    value: u64,
}

impl Clock {
    /// Given a 33-bit System Time Clock value, construct a new `Clock`
    /// value.
    pub const fn base(stc: u64) -> Self {
        Self { value: stc << 9 }
    }

    /// Return a new `Clock` value, setting the 9-bit extension to the
    /// specified value.
    pub fn with_ext(self, ext: u16) -> Self {
        Self {
            value: self.value & !0x1f | u64::from(ext),
        }
    }

    /// Convert a `Clock` value to seconds.
    #[expect(clippy::cast_precision_loss)]
    pub fn as_seconds(self) -> f64 {
        let base = (self.value >> 9) as f64;
        let ext = (self.value & 0x1F) as f64;
        (base + ext / 300.0) / 90000.0
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = self.as_seconds();
        let h = (s / 3600.0).trunc();
        s %= 3600.0;
        let m = (s / 60.0).trunc();
        s %= 60.0;
        write!(f, "{h}:{m:02}:{s:1.3}")
    }
}

/// Parse a 33-bit `Clock` value with 3 marker bits, consuming 36 bits.
pub fn clock(i: (&[u8], usize)) -> IResult<(&[u8], usize), Clock> {
    let marker = tag(0b1, 1usize);
    let hi_p = take(3usize);
    let mid_p = take(15usize);
    let lo_p = take(15usize);

    let (input, (hi, _, mid, _, lo, _)): ((&[u8], usize), (u64, _, u64, _, u64, _)) =
        (hi_p, &marker, mid_p, &marker, lo_p, &marker).parse(i)?;
    let clock = (hi << 30) | (mid << 15) | lo;
    Ok((input, Clock::base(clock)))
}

/// Parse a 33-bit `Clock` value plus a 9-bit extension and 4 marker bits,
/// consuming 46 bits.
pub fn clock_and_ext(input: (&[u8], usize)) -> IResult<(&[u8], usize), Clock> {
    let ext_bits = take(9u16);
    let clock_tag = tag(0b1, 1u8);
    let (input, (clock, ext, _)) = (clock, ext_bits, clock_tag).parse(input)?;
    Ok((input, clock.with_ext(ext)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_clock() {
        use nom::IResult;
        assert_eq!(
            clock((&[0x44, 0x02, 0xc4, 0x82, 0x04][..], 2)),
            IResult::Ok((
                (&[0x04][..], 6),
                Clock::base(0b0_0000_0000_0010_1100_0001_0000_0100_0000)
            ))
        );
        assert_eq!(
            clock_and_ext((&[0x44, 0x02, 0xc4, 0x82, 0x04, 0xa9][..], 2)),
            IResult::Ok((
                (&[][..], 0),
                Clock::base(0b0_0000_0000_0010_1100_0001_0000_0100_0000).with_ext(0b0_0101_0100)
            ))
        );
    }
}
