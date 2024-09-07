use commands::{Command, ParameterValue};

use crate::{coordinator::CoordinatorError, utils::slice_reader::SliceReader};
use std::{future::Future, io::Write};

pub mod commands;
pub mod subsystems;

pub const START_OF_FRAME: u8 = 0xFE;
pub const MAX_FRAME_SIZE: usize = 400;
pub const MAX_PAYLOAD_SIZE: usize = 255;
const MESSAGE_TYPE_MASK: u8 = 0b1110_0000;
const SUBSYSTEM_MASK: u8 = 0b0001_1111;

/*************************************************************************************************/
/*** TI Unified NPI Packet Format                                                              ***/
/***     SOF(1) + Length(2 or 1) + Type/Sub(1) + Cmd(1) + Payload(N) + FCS(1)                  ***/
/*************************************************************************************************/

#[derive(PartialEq, Clone)]
pub struct UnpiPacket<T> {
    pub len: LenType,
    pub type_subsystem: (MessageType, Subsystem),
    pub command: u8,
    pub payload: T,
    pub fcs: u8,
}

impl<T: AsRef<[u8]>> std::fmt::Debug for UnpiPacket<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnpiPacket")
            .field("len", &self.len)
            .field("type_subsystem", &self.type_subsystem)
            .field("command", &self.command)
            .field("payload", &&self.payload.as_ref()[0..self.len.size()])
            .field("fcs", &self.fcs)
            .finish()
    }
}

pub trait UnpiPacketSink<T> {
    fn write(
        &mut self,
        packet: &UnpiPacket<T>,
    ) -> impl Future<Output = Result<(), UnpiPacketError>>;
}

pub trait UnpiPacketSource<T> {
    fn read(&mut self) -> impl Future<Output = Result<UnpiPacket<T>, UnpiPacketError>>;
}

struct Wrapped<T>(T);

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LenType {
    OneByte(u8),
    TwoByte(u16),
}

impl LenType {
    pub fn to_le_bytes(&self) -> [u8; 2] {
        match self {
            LenType::OneByte(v) => [*v, 0],
            LenType::TwoByte(v) => v.to_le_bytes(),
        }
    }

    /// How many bytes it takes to store
    pub fn byte_size(&self) -> usize {
        match self {
            LenType::OneByte(_) => 1,
            LenType::TwoByte(_) => 2,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            LenType::OneByte(v) => *v as usize,
            LenType::TwoByte(v) => *v as usize,
        }
    }
}

impl From<LenType> for usize {
    fn from(v: LenType) -> usize {
        match v {
            LenType::OneByte(v) => v as usize,
            LenType::TwoByte(v) => v as usize,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LenTypeInfo {
    OneByte,
    TwoByte,
}

impl LenTypeInfo {
    /// How many bytes are used to represent the length
    pub fn byte_size(&self) -> usize {
        match self {
            LenTypeInfo::OneByte => 1,
            LenTypeInfo::TwoByte => 2,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UnpiPacketError {
    InvalidStartOfFrame(u8),
    InvalidFcs((u8, u8)),
    InvalidTypeSubsystem(u8),
    Parse,
    InvalidCommand,
    IoError,
    InvalidMessageType,
}

impl From<std::io::Error> for UnpiPacketError {
    fn from(_: std::io::Error) -> Self {
        UnpiPacketError::IoError
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MessageType {
    POLL,
    SREQ,
    AREQ,
    SRESP,
    RES0,
    RES1,
    RES2,
    RES3,
}

impl From<MessageType> for u8 {
    fn from(v: MessageType) -> u8 {
        match v {
            MessageType::POLL => 0,
            MessageType::SREQ => 1,
            MessageType::AREQ => 2,
            MessageType::SRESP => 3,
            MessageType::RES0 => 4,
            MessageType::RES1 => 5,
            MessageType::RES2 => 6,
            MessageType::RES3 => 7,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Subsystem {
    Res0,
    Sys,
    Mac,
    Nwk,
    Af,
    Zdo,
    Sapi,
    Util,
    Dbg,
    App,
    Rcaf,
    Rcn,
    RcnClient,
    Boot,
    Ziptest,
    Debug,
    Peripherals,
    Nfc,
    PbNwkMgr,
    PbGw,
    PbOtaMgr,
    BleSpnp,
    BleHci,
    Resv01,
    Resv02,
    Resv03,
    Resv04,
    Resv05,
    Resv06,
    Resv07,
    Resv08,
    SrvCtr,
}

impl From<Subsystem> for u8 {
    fn from(v: Subsystem) -> u8 {
        match v {
            Subsystem::Res0 => 0,
            Subsystem::Sys => 1,
            Subsystem::Mac => 2,
            Subsystem::Nwk => 3,
            Subsystem::Af => 4,
            Subsystem::Zdo => 5,
            Subsystem::Sapi => 6,
            Subsystem::Util => 7,
            Subsystem::Dbg => 8,
            Subsystem::App => 9,
            Subsystem::Rcaf => 10,
            Subsystem::Rcn => 11,
            Subsystem::RcnClient => 12,
            Subsystem::Boot => 13,
            Subsystem::Ziptest => 14,
            Subsystem::Debug => 15,
            Subsystem::Peripherals => 16,
            Subsystem::Nfc => 17,
            Subsystem::PbNwkMgr => 18,
            Subsystem::PbGw => 19,
            Subsystem::PbOtaMgr => 20,
            Subsystem::BleSpnp => 21,
            Subsystem::BleHci => 22,
            Subsystem::Resv01 => 23,
            Subsystem::Resv02 => 24,
            Subsystem::Resv03 => 25,
            Subsystem::Resv04 => 26,
            Subsystem::Resv05 => 27,
            Subsystem::Resv06 => 28,
            Subsystem::Resv07 => 29,
            Subsystem::Resv08 => 30,
            Subsystem::SrvCtr => 31,
        }
    }
}

impl TryFrom<u8> for MessageType {
    type Error = UnpiPacketError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::POLL),
            1 => Ok(MessageType::SREQ),
            2 => Ok(MessageType::AREQ),
            3 => Ok(MessageType::SRESP),
            4 => Ok(MessageType::RES0),
            5 => Ok(MessageType::RES1),
            6 => Ok(MessageType::RES2),
            7 => Ok(MessageType::RES3),
            _ => Err(UnpiPacketError::InvalidMessageType),
        }
    }
}

impl<'a> TryFrom<(&'a [u8], LenTypeInfo)> for UnpiPacket<&'a [u8]> {
    type Error = UnpiPacketError;

    /// Type safe constructor for UnpiPacket with dependence of len_type_info
    fn try_from(
        (data, len_type_info): (&'a [u8], LenTypeInfo),
    ) -> Result<UnpiPacket<&'a [u8]>, Self::Error> {
        let mut data = SliceReader(data);
        let sof = data.read_u8()?;
        if sof != START_OF_FRAME {
            return Err(UnpiPacketError::InvalidStartOfFrame(sof));
        }
        let len = match len_type_info {
            LenTypeInfo::OneByte => LenType::OneByte(data.read_u8()?),
            LenTypeInfo::TwoByte => LenType::TwoByte(data.read_u16_le()?),
        };
        let type_subsystem = data.read_u8()?;
        let command = data.read_u8()?;
        let payload = data.subslice_exact(len.size())?;
        let fcs = data.read_u8()?;
        let p = UnpiPacket {
            len,
            type_subsystem: Wrapped(type_subsystem)
                .try_into()
                .map_err(|_| UnpiPacketError::InvalidTypeSubsystem(type_subsystem))?,
            command,
            payload,
            fcs,
        };
        let checksum = p.checksum()?;
        if checksum == p.fcs {
            Ok(p)
        } else {
            Err(UnpiPacketError::InvalidFcs((p.fcs, checksum)))
        }
    }
}

impl From<(MessageType, Subsystem)> for Wrapped<u8> {
    fn from(value: (MessageType, Subsystem)) -> Wrapped<u8> {
        Wrapped(Into::<u8>::into(value.0) << 5 | Into::<u8>::into(value.1))
    }
}

impl TryFrom<Wrapped<u8>> for (MessageType, Subsystem) {
    type Error = UnpiPacketError;

    fn try_from(value: Wrapped<u8>) -> Result<Self, Self::Error> {
        let v = value.0;
        let message_type = (v & MESSAGE_TYPE_MASK) >> 5;
        let subsystem = v & SUBSYSTEM_MASK;
        Ok((
            message_type.try_into()?,
            subsystem
                .try_into()
                .map_err(|_| UnpiPacketError::InvalidTypeSubsystem(v))?,
        ))
    }
}
impl<'a> UnpiPacket<Vec<u8>> {
    pub fn from_payload_owned(
        (payload, len_type_info): (&'a [u8], LenTypeInfo),
        type_subsystem: (MessageType, Subsystem),
        command: u8,
    ) -> Result<UnpiPacket<Vec<u8>>, std::io::Error> {
        let h = UnpiPacket {
            len: match len_type_info {
                LenTypeInfo::OneByte => LenType::OneByte(payload.len() as u8),
                LenTypeInfo::TwoByte => LenType::TwoByte(payload.len() as u16),
            },
            type_subsystem,
            command,
            payload: payload.to_vec(),
            fcs: 0,
        };
        let fcs = h.checksum()?;
        Ok(UnpiPacket { fcs, ..h })
    }
    pub fn from_command_owned(
        len_type_info: LenTypeInfo,
        type_subsystem: (MessageType, Subsystem),
        parameters: &[(&'static str, ParameterValue)],
        command: &Command,
    ) -> Result<UnpiPacket<Vec<u8>>, CoordinatorError> {
        let mut payload = vec![0u8; MAX_PAYLOAD_SIZE];
        let written = command.fill_and_write(parameters, &mut payload)?;
        let h = UnpiPacket {
            len: match len_type_info {
                LenTypeInfo::OneByte => LenType::OneByte(written as u8),
                LenTypeInfo::TwoByte => LenType::TwoByte(written as u16),
            },
            type_subsystem,
            command: command.id,
            payload: payload.to_vec(),
            fcs: 0,
        };
        let fcs = h.checksum()?;
        Ok(UnpiPacket { fcs, ..h })
    }
}

impl<'a> UnpiPacket<&'a [u8]> {
    pub fn from_payload(
        (payload, len_type_info): (&'a [u8], LenTypeInfo),
        type_subsystem: (MessageType, Subsystem),
        command: u8,
    ) -> Result<UnpiPacket<&'a [u8]>, std::io::Error> {
        let h = UnpiPacket {
            len: match len_type_info {
                LenTypeInfo::OneByte => LenType::OneByte(payload.len() as u8),
                LenTypeInfo::TwoByte => LenType::TwoByte(payload.len() as u16),
            },
            type_subsystem,
            command,
            payload,
            fcs: 0,
        };
        let fcs = h.checksum()?;
        Ok(UnpiPacket { fcs, ..h })
    }
}

impl<T> UnpiPacket<T>
where
    T: AsRef<[u8]>,
{
    pub fn to_bytes(&self, mut output: &mut [u8]) -> Result<usize, std::io::Error> {
        let len = output.len();
        output.write_all(&[START_OF_FRAME])?;
        match self.len {
            LenType::OneByte(_) => output.write_all(&self.len.to_le_bytes()[0..1])?,
            LenType::TwoByte(_) => output.write_all(&self.len.to_le_bytes())?,
        };
        let payload_len = self.len.size();
        output.write_all(&[Into::<Wrapped<u8>>::into(self.type_subsystem.clone()).0])?;
        output.write_all(&[self.command])?;
        output.write_all(&self.payload.as_ref()[0..payload_len])?;
        output.write_all(&[self.fcs])?;
        Ok(len - output.len())
    }

    #[allow(unused)]
    //https://github.com/Mr-Markus/unpi-net/blob/master/src/UnpiNet/Packet.cs
    pub fn checksum_buffer(buf: &[u8]) -> u8 {
        let mut fcs: u8 = 0x00;

        for &byte in buf {
            fcs ^= byte;
        }

        fcs
    }

    pub fn checksum(&self) -> Result<u8, std::io::Error> {
        let output: &mut [u8] = &mut [0u8; MAX_FRAME_SIZE];
        let len = self.to_bytes(output)?;
        Ok(Self::checksum_buffer(&output[1..(len - 1)]))
    }
}

impl TryFrom<u8> for Subsystem {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value & SUBSYSTEM_MASK {
            0 => Ok(Subsystem::Res0),
            1 => Ok(Subsystem::Sys),
            2 => Ok(Subsystem::Mac),
            3 => Ok(Subsystem::Nwk),
            4 => Ok(Subsystem::Af),
            5 => Ok(Subsystem::Zdo),
            6 => Ok(Subsystem::Sapi),
            7 => Ok(Subsystem::Util),
            8 => Ok(Subsystem::Dbg),
            9 => Ok(Subsystem::App),
            10 => Ok(Subsystem::Rcaf),
            11 => Ok(Subsystem::Rcn),
            12 => Ok(Subsystem::RcnClient),
            13 => Ok(Subsystem::Boot),
            14 => Ok(Subsystem::Ziptest),
            15 => Ok(Subsystem::Debug),
            16 => Ok(Subsystem::Peripherals),
            17 => Ok(Subsystem::Nfc),
            18 => Ok(Subsystem::PbNwkMgr),
            19 => Ok(Subsystem::PbGw),
            20 => Ok(Subsystem::PbOtaMgr),
            21 => Ok(Subsystem::BleSpnp),
            22 => Ok(Subsystem::BleHci),
            23 => Ok(Subsystem::Resv01),
            24 => Ok(Subsystem::Resv02),
            25 => Ok(Subsystem::Resv03),
            26 => Ok(Subsystem::Resv04),
            27 => Ok(Subsystem::Resv05),
            28 => Ok(Subsystem::Resv06),
            29 => Ok(Subsystem::Resv07),
            30 => Ok(Subsystem::Resv08),
            31 => Ok(Subsystem::SrvCtr),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //some test cases from https://github.com/shimmeringbee/unpi/blob/main/frame_test.go

    #[test]
    pub fn test_unpi_empty() {
        let data = [0xFEu8, 0x00, 0x25, 0x37, 0x12];
        let packet = UnpiPacket::try_from((&data[..], LenTypeInfo::OneByte)).unwrap();
        assert_eq!(packet.type_subsystem, (MessageType::SREQ, Subsystem::Zdo));
        assert_eq!(packet.command, 0x37);
        assert_eq!(packet.payload.len(), 0);
    }

    #[test]
    pub fn test_unpi_empty_wrong_checksum() {
        let data = [0xFEu8, 0x00, 0x25, 0x37, 0x01];
        let packet = UnpiPacket::try_from((&data[..], LenTypeInfo::OneByte));
        assert_eq!(packet, Err(UnpiPacketError::InvalidFcs((0x01, 0x12))));
    }

    #[test]
    pub fn test_unpi_payload() {
        let data = [0xfe, 0x02, 0x25, 0x37, 0x55, 0xdd, 0x98];
        let packet = UnpiPacket::try_from((&data[..], LenTypeInfo::OneByte)).unwrap();
        assert_eq!(packet.type_subsystem, (MessageType::SREQ, Subsystem::Zdo));
        assert_eq!(packet.command, 0x37);
        assert_eq!(packet.payload, &[0x55, 0xdd]);
    }

    #[test]
    pub fn test_unpi_payload_to_from_bytes() {
        let data = [0xfe, 0x02, 0x25, 0x37, 0x55, 0xdd, 0x98];
        let packet = UnpiPacket::try_from((&data[..], LenTypeInfo::OneByte)).unwrap();
        let mut output: &mut [u8] = &mut [0u8; MAX_FRAME_SIZE];
        let len = packet.to_bytes(&mut output).unwrap();
        assert_eq!(&output[0..len], &data[..]);
    }

    #[test]
    pub fn test_unpi_to_bytes() {
        let packet: UnpiPacket<&[u8]> = UnpiPacket::from_payload(
            (&[0x55u8, 0xdd], LenTypeInfo::OneByte),
            (MessageType::SREQ, Subsystem::Zdo),
            0x37,
        )
        .unwrap();
        let output = &mut [0u8; MAX_FRAME_SIZE];
        let len = packet.to_bytes(&mut output.as_mut()).unwrap();
        assert_eq!(&output[0..len], &[0xFE, 0x02, 0x25, 0x37, 0x55, 0xdd, 0x98]);
    }

    #[test]
    pub fn test_unpi_double_len() {
        let data = [0xFEu8, 0x04, 0x00, 0x25, 0x04, 0x01, 0x02, 0x03, 0x04, 0x21];
        let packet = UnpiPacket::try_from((&data[..], LenTypeInfo::TwoByte)).unwrap();
        let checksum = UnpiPacket::<&[u8]>::checksum_buffer(&data[1..data.len() - 1]);
        assert_eq!(packet.len, LenType::TwoByte(0x04));
        assert_eq!(packet.type_subsystem, (MessageType::SREQ, Subsystem::Zdo));
        assert_eq!(packet.command, 0x04);
        assert_eq!(packet.payload, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(checksum, packet.fcs);
        assert_eq!(checksum, packet.checksum().unwrap());
    }

    #[test]
    pub fn test_unpi_double_len_wrong_checksum() {
        let data = [0xFEu8, 0x04, 0x00, 0x25, 0x04, 0x01, 0x02, 0x03, 0x04, 0x02];
        let packet = UnpiPacket::try_from((&data[..], LenTypeInfo::TwoByte));
        assert_eq!(packet, Err(UnpiPacketError::InvalidFcs((0x02, 0x21))));
    }

    #[test]
    pub fn test_unpi_double_len_to_from_bytes() {
        let data = [0xFEu8, 0x04, 0x00, 0x25, 0x04, 0x01, 0x02, 0x03, 0x04, 0x21];
        let packet = UnpiPacket::try_from((&data[..], LenTypeInfo::TwoByte)).unwrap();
        let mut output: &mut [u8] = &mut [0u8; MAX_FRAME_SIZE];
        let len = packet.to_bytes(&mut output).unwrap();
        assert_eq!(&output[0..len], &data[..])
    }
}
