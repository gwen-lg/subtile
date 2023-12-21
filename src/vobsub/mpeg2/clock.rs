use std::fmt;

use nom::{do_parse, named, tag_bits, take_bits};

/// This represents the 90kHz, 33-bit [System Time Clock][STC] (STC) and
/// the 9-bit STC extension value, which represents 1/300th of a tick.
///
/// [STC]: http://www.bretl.com/mpeghtml/STC.HTM
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Clock {
    value: u64,
}

impl Clock {
    /// Given a 33-bit System Time Clock value, construct a new `Clock`
    /// value.
    pub fn base(stc: u64) -> Clock {
        Clock { value: stc << 9 }
    }

    /// Return a new `Clock` value, setting the 9-bit extension to the
    /// specified value.
    pub fn with_ext(&self, ext: u16) -> Clock {
        Clock {
            value: self.value & !0x1f | u64::from(ext),
        }
    }

    /// Convert a `Clock` value to seconds.
    pub fn as_seconds(&self) -> f64 {
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
        write!(f, "{}:{:02}:{:1.3}", h, m, s)
    }
}

/// Parse a 33-bit `Clock` value with 3 marker bits, consuming 36 bits.
named!(pub clock<(&[u8], usize), Clock>,
    do_parse!(
        // Bits 32..30.
        hi: take_bits!(u64, 3) >>
        // Marker bit.
        tag_bits!(u8, 1, 0b1) >>
        // Bits 29..15.
        mid: take_bits!(u64, 15) >>
        // Marker bit.
        tag_bits!(u8, 1, 0b1) >>
        // Bits 14..0.
        lo: take_bits!(u64, 15) >>
        // Marker bit.
        tag_bits!(u8, 1, 0b1) >>
        (Clock::base(hi << 30 | mid << 15 | lo))
    )
);

/// Parse a 33-bit `Clock` value plus a 9-bit extension and 4 marker bits,
/// consuming 46 bits.
named!(pub clock_and_ext<(&[u8], usize), Clock>,
    do_parse!(
        clock: call!(clock) >>
        ext: take_bits!(u16, 9) >>
        // Marker bit.
        tag_bits!(u8, 1, 0b1) >>
        (clock.with_ext(ext))
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_clock() {
        use nom::IResult;
        assert_eq!(
            clock((&[0x44, 0x02, 0xc4, 0x82, 0x04][..], 2)),
            IResult::Done(
                (&[0x04][..], 6),
                Clock::base(0b0_0000_0000_0010_1100_0001_0000_0100_0000)
            )
        );
        assert_eq!(
            clock_and_ext((&[0x44, 0x02, 0xc4, 0x82, 0x04, 0xa9][..], 2)),
            IResult::Done(
                (&[][..], 0),
                Clock::base(0b0_0000_0000_0010_1100_0001_0000_0100_0000).with_ext(0b0_0101_0100)
            )
        );
    }
}
