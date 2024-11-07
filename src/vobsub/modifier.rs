use std::{
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
};

use super::{mpeg2::ps, VobSubError};

pub struct DataAccessor<'a> {
    pub data: &'a mut [u8],
}

impl<'a> DataAccessor<'a> {
    pub fn shift(&'a mut self, bytes: usize) {
        self.data = &mut self.data[bytes..];
    }

    pub fn clear(&'a mut self) {
        self.data = &mut [];
    }
}

impl<'a> Borrow<[u8]> for DataAccessor<'a> {
    fn borrow(&self) -> &[u8] {
        self.data
    }
}

impl<'a> BorrowMut<[u8]> for DataAccessor<'a> {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.data
    }
}

//impl<'a> ParseError<&[u8]> for nom::error::Error<DataAccessor<'_>> {
//     fn from_error_kind(input: &[u8], kind: nom::error::ErrorKind) -> Self {
//         todo!()
//         //&[u8]::from_error_kind(input, kind)
//     }

//     fn append(input: &[u8], kind: nom::error::ErrorKind, other: Self) -> Self {
//         todo!()
//     }
//}

/// Implement a tool to modify the `VobSub` data inplace (or during streaming).
pub struct VobsubModifier<'a, Modifier> {
    pes_packets: ps::PesPackets<'a>,
    phantom_data: PhantomData<Modifier>,
}

impl<'a, Modifier> VobsubModifier<'a, Modifier> {
    /// To update a `vobsub` (.sub) file content.
    #[must_use]
    pub fn new(input: &'a mut [u8]) -> Self {
        let in_data = DataAccessor { data: input };
        Self {
            pes_packets: ps::pes_packets(in_data),
            phantom_data: PhantomData,
        }
    }

    /// Apply a time shift on all subtitles of a `VobSub`.
    ///
    /// # Errors
    ///
    /// Will return `Err` if wasn't able to decode the the input.
    pub fn time_shift(&mut self) -> Result<(), VobSubError> {
        profiling::scope!("VobsubModifier process");

        self.pes_packets.try_for_each(|pes_packet| {
            let pes_packet = pes_packet?;

            //    pub ps_header: Header,
            // pub pes_packet: pes::Packet<'a>,
            if let Some(pts_dts) = pes_packet.pes_packet.header_data.pts_dts {
                let seconds = pts_dts.pts.as_seconds();
            }

            Ok(())
        })
    }
}
