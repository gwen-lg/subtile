//! # MPEG-2 Packetized Elementary Streams (`PES`)
//!
//! These packets are nested inside the MPEG-2 Program Stream packets found
//! in a `*.sub` file.

use nom::{
    bits::{
        self, bits,
        complete::{tag as tag_bits, take},
    },
    branch::alt,
    bytes::complete::tag as tag_bytes,
    combinator::{map, rest, value},
    multi::length_value,
    number::complete::{be_u16, be_u8},
    //do_parse, length_value, named, rest,
    IResult,
    Parser as _,
};
use std::fmt;

use super::clock::{clock, Clock};
use crate::util::BytesFormatter;

/// Possible combinations of `PTS` and `DTS` data which might appear inside a
/// `PES` header.
///
/// See the [`PES` header documentation][PES] for details.
///
/// [PES]: http://dvd.sourceforge.net/dvdinfo/pes-hdr.html
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PtsDtsFlags {
    /// No time stamps.
    #[default]
    None,
    /// Presentation Time Stamp only.
    Pts,
    /// Presentation and Decode Time Stamps.
    PtsDts,
}

/// Parse `PTS` & `DTS` flags in a `PES` packet header.  Consumes two bits.
fn pts_dts_flags(input: (&[u8], usize)) -> IResult<(&[u8], usize), PtsDtsFlags> {
    alt((
        value(PtsDtsFlags::None, tag_bits(0b00, 2u8)),
        value(PtsDtsFlags::Pts, tag_bits(0b10, 2u8)),
        value(PtsDtsFlags::PtsDts, tag_bits(0b11, 2u8)),
    ))
    .parse(input)
}

/// Presentation and Decode Time Stamps, if available.
#[derive(Debug, PartialEq, Eq)]
pub struct PtsDts {
    /// Presentation Time Stamp.
    pub pts: Clock,
    /// Decode Time Stamp.
    pub dts: Option<Clock>,
}

/// Helper for `pts_dts`.  Parses the PTS-only case.
fn pts_only(input: &[u8]) -> IResult<&[u8], PtsDts> {
    bits(|input| {
        let tag_parse = tag_bits(0b0010, 4u8);
        let (input, (_, pts)) = (tag_parse, clock).parse(input)?;
        Ok((input, PtsDts { pts, dts: None }))
    })(input)
}

/// Helper for `pts_dts`.  Parses the `PTS` and `DTS` case.
fn pts_and_dts(input: &[u8]) -> IResult<&[u8], PtsDts> {
    bits(|input| {
        let parse_tag = tag_bits(0b0010, 4u8);
        let (input, (_, pts, _, dts)): ((&[u8], usize), (_, _, _, _)) =
            (&parse_tag, clock, &parse_tag, clock).parse(input)?;
        Ok((
            input,
            PtsDts {
                pts,
                dts: Some(dts),
            },
        ))
    })(input)
}

/// Parse a `PtsDts` value in the format specified by `flags`.
fn pts_dts(i: &[u8], flags: PtsDtsFlags) -> IResult<&[u8], Option<PtsDts>> {
    match flags {
        PtsDtsFlags::None => IResult::Ok((i, None)),
        PtsDtsFlags::Pts => pts_only(i).map(|(i, pts)| (i, Some(pts))),
        PtsDtsFlags::PtsDts => pts_and_dts(i).map(|(i, ptsdts)| (i, Some(ptsdts))),
    }
}

/// Flags specifying which header data fields are present.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct HeaderDataFlags {
    pub pts_dts_flags: PtsDtsFlags,
    pub escr_flag: bool,
    pub es_rate_flag: bool,
    pub dsm_trick_mode_flag: bool,
    pub additional_copy_info_flag: bool,
    pub crc_flag: bool,
    pub extension_flag: bool,
}

/// Deserialize a single Boolean flag bit.
fn bool_flag(input: (&[u8], usize)) -> IResult<(&[u8], usize), bool> {
    map(|input| bits::complete::take(1u8)(input), |b: u8| b == 1).parse(input)
}

/// Deserialize `HeaderDataFlags`
fn header_data_flags(input: &[u8]) -> IResult<&[u8], HeaderDataFlags> {
    bits(|input| {
        let (
            input,
            (
                pts_dts_flags,
                escr_flag,
                es_rate_flag,
                dsm_trick_mode_flag,
                additional_copy_info_flag,
                crc_flag,
                extension_flag,
            ),
        ) = (
            pts_dts_flags,
            bool_flag,
            bool_flag,
            bool_flag,
            bool_flag,
            bool_flag,
            bool_flag,
        )
            .parse(input)?;
        Ok((
            input,
            HeaderDataFlags {
                pts_dts_flags,
                escr_flag,
                es_rate_flag,
                dsm_trick_mode_flag,
                additional_copy_info_flag,
                crc_flag,
                extension_flag,
            },
        ))
    })(input)
}

/// Header data fields.
#[non_exhaustive]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct HeaderData {
    pub flags: HeaderDataFlags,
    pub pts_dts: Option<PtsDts>,
}

/// Parse `PES` header data, including the preceding flags and length bytes.
fn header_data(input: &[u8]) -> IResult<&[u8], HeaderData> {
    // Grab the flags from our flag byte with header_data_flags.
    let (input, flags) = header_data_flags(input)?;

    // Grab a single length byte, read that many bytes, and recursively
    // call `header_data_fields` to do the actual parse.  This ensures
    // that if `header_data_fields` doesn't parse all the header data,
    // we discard the rest before continuing.
    let (input, pts_dts) =
        length_value(be_u8, |input| pts_dts(input, flags.pts_dts_flags)).parse(input)?;
    Ok((input, HeaderData { flags, pts_dts }))
}

/// A [Packetized Elementary Stream][pes] header, not including the
/// `HeaderData` information (which is parsed separately).
///
/// [pes]: http://dvd.sourceforge.net/dvdinfo/pes-hdr.html
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Header {
    pub scrambling_control: u8,
    pub priority: bool,
    pub data_alignment_indicator: bool,
    pub copyright: bool,
    pub original: bool,
}

/// Parse the first `PES` header byte after the length.
fn header(input: &[u8]) -> IResult<&[u8], Header> {
    bits(|input| {
        let tag_parse = tag_bits(0b10, 2u8);
        let take_scrambling = take(2u8);
        let (
            input,
            (_, scrambling_control, priority, data_alignment_indicator, copyright, original),
        ) = (
            tag_parse,
            take_scrambling,
            bool_flag,
            bool_flag,
            bool_flag,
            bool_flag,
        )
            .parse(input)?;
        Ok((
            input,
            Header {
                scrambling_control,
                priority,
                data_alignment_indicator,
                copyright,
                original,
            },
        ))
    })(input)
}

/// A [Packetized Elementary Stream][pes] packet.
///
/// [pes]: http://dvd.sourceforge.net/dvdinfo/pes-hdr.html
#[derive(PartialEq, Eq)]
pub struct Packet<'a> {
    pub header: Header,
    pub header_data: HeaderData,
    pub substream_id: u8,
    pub data: &'a [u8],
}

impl fmt::Debug for Packet<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Packet")
            .field("header", &self.header)
            .field("header_data", &self.header_data)
            .field("substream_id", &self.substream_id)
            .field("data", &BytesFormatter(self.data))
            .finish()
    }
}

fn packet_helper(input: &[u8]) -> IResult<&[u8], Packet<'_>> {
    let (input, (header, header_data, substream_id, data)) =
        (header, header_data, be_u8, rest).parse(input)?;
    Ok((
        input,
        Packet {
            header,
            header_data,
            substream_id,
            data,
        },
    ))
}

pub fn packet(input: &[u8]) -> IResult<&[u8], Packet<'_>> {
    const PACKET_TAG: &[u8] = &[0x00, 0x00, 0x01, 0xbd];
    let packet_tag = tag_bytes(PACKET_TAG);
    let packet_data = length_value(be_u16, packet_helper);
    let (input, (_, packet)) = (packet_tag, packet_data).parse(input)?;

    Ok((input, packet))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pts_dts_flags() {
        assert_eq!(
            pts_dts_flags((&[0b00][..], 6)),
            IResult::Ok(((&[][..], 0), PtsDtsFlags::None))
        );
        assert_eq!(
            pts_dts_flags((&[0b10][..], 6)),
            IResult::Ok(((&[][..], 0), PtsDtsFlags::Pts))
        );
        assert_eq!(
            pts_dts_flags((&[0b11][..], 6)),
            IResult::Ok(((&[][..], 0), PtsDtsFlags::PtsDts))
        );
    }

    #[test]
    fn parse_pts_dts() {
        assert_eq!(
            pts_dts(&[][..], PtsDtsFlags::None),
            IResult::Ok((&[][..], None))
        );
        assert_eq!(
            pts_dts(&[0x21, 0x00, 0xab, 0xe9, 0xc1][..], PtsDtsFlags::Pts),
            IResult::Ok((
                &[][..],
                Some(PtsDts {
                    pts: Clock::base(2_815_200),
                    dts: None,
                })
            ))
        );
    }

    #[test]
    fn parse_header_data_flags() {
        assert_eq!(
            header_data_flags(&[0x80][..]),
            IResult::Ok((
                &[][..],
                HeaderDataFlags {
                    pts_dts_flags: PtsDtsFlags::Pts,
                    ..HeaderDataFlags::default()
                }
            ))
        );
    }

    #[test]
    fn parse_header_data() {
        assert_eq!(
            header_data(&[0x00, 0x00][..]),
            IResult::Ok((&[][..], HeaderData::default()))
        );
        assert_eq!(
            header_data(&[0x80, 0x05, 0x21, 0x00, 0xab, 0xe9, 0xc1][..]),
            IResult::Ok((
                &[][..],
                HeaderData {
                    flags: HeaderDataFlags {
                        pts_dts_flags: PtsDtsFlags::Pts,
                        ..HeaderDataFlags::default()
                    },
                    pts_dts: Some(PtsDts {
                        pts: Clock::base(2_815_200),
                        dts: None,
                    }),
                    ..HeaderData::default()
                }
            ))
        );
    }

    #[test]
    fn parse_packet() {
        let input = &[
            0x00, 0x00, 0x01, 0xbd, 0x00, 0x10, 0x81, 0x80, 0x05, 0x21, 0x00, 0xab, 0xe9, 0xc1,
            0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff,
        ][..];

        let expected = Packet {
            header: Header {
                original: true,
                ..Header::default()
            },
            header_data: HeaderData {
                flags: HeaderDataFlags {
                    pts_dts_flags: PtsDtsFlags::Pts,
                    ..HeaderDataFlags::default()
                },
                pts_dts: Some(PtsDts {
                    pts: Clock::base(2_815_200),
                    dts: None,
                }),
                ..HeaderData::default()
            },
            substream_id: 0x20,
            data: &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        };

        assert_eq!(packet(input), IResult::Ok((&[0xff][..], expected)));
    }
}
