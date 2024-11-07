use std::marker::PhantomData;

use super::{mpeg2::ps, VobSubError};

/// Implement a tool to modify the `VobSub` data inplace (or during streaming).
pub struct VobsubModifier<'a, Modifier> {
    pes_packets: ps::PesPackets<'a>,
    phantom_data: PhantomData<Modifier>,
}

impl<'a, Modifier> VobsubModifier<'a, Modifier> {
    /// To update a `vobsub` (.sub) file content.
    #[must_use]
    pub fn new(input: &'a mut [u8]) -> Self {
        Self {
            pes_packets: ps::pes_packets(input),
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
