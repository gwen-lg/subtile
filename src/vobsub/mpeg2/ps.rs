//! # MPEG-2 Program Streams (PS)
//!
//! This is the container format used at the top-level of a `*.sub` file.

use log::{debug, trace, warn};
use nom::{
    bits::{
        bits,
        complete::{tag as tag_bits, take as take_bits},
    },
    bytes::complete::tag as tag_bytes,
    sequence::Tuple,
    IResult,
};
use std::{borrow::Borrow, fmt};

use super::{
    clock::{clock_and_ext, Clock},
    pes,
};
use crate::vobsub::{modifier::DataAccessor, NomError, VobSubError};

/// A parsed [MPEG-2 Program Stream header][MPEG-PS] (MPEG-PS).
///
/// [MPEG-PS]: https://en.wikipedia.org/wiki/MPEG_program_stream
#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    /// The System Clock Reference (`SCR`) and `SCR` extension field.
    pub scr: Clock,
    /// The bit rate, in units of 50 bytes per second.
    pub bit_rate: u32,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[PS packet @ {}, {} kbps]",
            self.scr,
            (self.bit_rate * 50 * 8) / 1024
        )
    }
}

/// Parse a Program Stream header.
pub fn header(input: DataAccessor) -> IResult<DataAccessor, Header> {
    // Sync bytes.
    let tag1 = tag_bytes(&[0x00, 0x00, 0x01, 0xba]);

    // 10-byte header.
    let header_parse = bits(|input| {
        // MPEG-2 version tag.
        let tag_mpeg2 = tag_bits(0b01, 2u8);
        // Bit rate
        let bit_rate = take_bits(22u32);
        // Marker bits.
        let marker_bits = tag_bits(0b11, 2u8);
        // Reserved.
        let reserved = take_bits::<_, u8, u8, nom::error::Error<(&[u8], usize)>>(5u8);
        // Number of bytes of stuffing.
        let stuffing_length =
            take_bits::<_, usize, usize, nom::error::Error<(&[u8], usize)>>(3usize);

        // clock_and_ext: System Clock Reference.
        let (input, (_, scr, bit_rate, _, _, stuffing_length)) = (
            tag_mpeg2,
            clock_and_ext,
            bit_rate,
            marker_bits,
            reserved,
            stuffing_length,
        )
            .parse(input)?;

        // Stuffing bytes.  We just want to ignore these, but use a
        // large enough type to prevent overflow panics when
        // fuzzing.
        let (input, _) = take_bits::<_, u64, _, _>(stuffing_length * 8)(input)?;
        Ok((input, Header { scr, bit_rate }))
    });

    let (input, (_, header)) = (tag1, header_parse).parse(input)?;
    Ok((input, header))
}

/// A [Packetized Elementary Stream][pes] packet with a Program Stream
/// header.
///
/// [pes]: http://dvd.sourceforge.net/dvdinfo/pes-hdr.html
#[derive(Debug, PartialEq, Eq)]
pub struct PesPacket<'a> {
    pub ps_header: Header,
    pub pes_packet: pes::Packet<'a>,
}

/// Parse a Program Stream packet and the following `PES` packet.
pub fn pes_packet(input: DataAccessor) -> IResult<DataAccessor, PesPacket> {
    let (input, (ps_header, pes_packet)) = (header, pes::packet).parse(input)?;
    Ok((
        input,
        PesPacket {
            ps_header,
            pes_packet,
        },
    ))
}

/// An iterator over all the `PES` packets in an MPEG-2 Program Stream.
pub struct PesPackets<'a> {
    /// The remaining input to parse.
    remaining: DataAccessor<'a>,
}

impl<'a> Iterator for PesPackets<'a> {
    type Item = Result<PesPacket<'a>, VobSubError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Search for the start of a ProgramStream packet.
            let needle = &[0x00, 0x00, 0x01, 0xba];
            let start = {
                let slice: &[u8] = self.remaining.borrow();
                slice
                    .windows(needle.len())
                    .position(|window| needle == window)
            };

            if let Some(start) = start {
                // We found the start, so try to parse it.
                self.remaining.shift(start); //= &self.remaining[start..];
                match pes_packet(self.remaining) {
                    // We found a packet!
                    IResult::Ok((remaining, packet)) => {
                        self.remaining = remaining;
                        trace!("Decoded packet {:?}", &packet);
                        return Some(Ok(packet));
                    }

                    IResult::Err(err) => match err {
                        // We have only a partial packet, and we hit the end of our
                        // data.
                        nom::Err::Incomplete(needed) => {
                            self.remaining.clear(); // = &[];
                            warn!("Incomplete packet, need: {needed:?}");
                            return Some(Err(VobSubError::PESPacket(NomError::IncompleteInput(
                                needed,
                            ))));
                        }
                        // We got something that looked like a packet but
                        // wasn't parseable.  Log it and keep trying.
                        nom::Err::Error(err) | nom::Err::Failure(err) => {
                            self.remaining.shift(needle.len()); //= &self.remaining[needle.len()..];
                            debug!("Skipping packet {:?}", &err);
                        }
                    },
                }
            } else {
                // We didn't find the start of a packet.
                self.remaining.clear(); // = &[];
                trace!("Reached end of data");
                return None;
            }
        }
    }
}

/// Iterate over all the `PES` packets in an MPEG-2 Program Stream (or at
/// least those which contain subtitles).
pub const fn pes_packets(input: DataAccessor) -> PesPackets {
    PesPackets { remaining: input }
}
