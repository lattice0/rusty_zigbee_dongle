pub const MAX_FRAME_SIZE: usize = 255;
const MESSAGE_TYPE_MASK: u8 = 0b1110_0000;
const SUBSYSTEM_MASK: u8 = 0b0001_1111;

pub struct Unpi {}

/*************************************************************************************************/
/*** TI Unified NPI Packet Format                                                              ***/
/***     SOF(1) + Length(2/1) + Type/Sub(1) + Cmd(1) + Payload(N) + FCS(1)                     ***/
/*************************************************************************************************/
#[derive(Debug, PartialEq)]
pub struct UnpiPacket<'a> {
    pub len: u16,
    pub type_subsystem: (MessageType, Subsystem),
    pub command: u8,
    pub payload: &'a [u8],
    pub fcs: u8,
}

#[derive(Debug, PartialEq)]
pub enum UnpiHeaderError {
    InvalidStartOfFrame,
    InvalidFcs,
    InvalidTypeSubsystem,
    Parse,
    InvalidCommand,
}

#[derive(Debug, PartialEq, Clone)]
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

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        let r = match self {
            MessageType::POLL => 0,
            MessageType::SREQ => 1,
            MessageType::AREQ => 2,
            MessageType::SRESP => 3,
            MessageType::RES0 => 4,
            MessageType::RES1 => 5,
            MessageType::RES2 => 6,
            MessageType::RES3 => 7,
        };
        r
    }
}

impl Unpi {
    pub fn new() -> Unpi {
        Unpi {}
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

impl Into<u8> for Subsystem {
    fn into(self) -> u8 {
        let r = match self {
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
        };
        r
    }
}

impl TryFrom<u8> for MessageType {
    type Error = UnpiHeaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        println!("message try from {:?}", value);
        match value {
            0 => Ok(MessageType::POLL),
            1 => Ok(MessageType::SREQ),
            2 => Ok(MessageType::AREQ),
            3 => Ok(MessageType::SRESP),
            4 => Ok(MessageType::RES0),
            5 => Ok(MessageType::RES1),
            6 => Ok(MessageType::RES2),
            7 => Ok(MessageType::RES3),
            _ => Err(UnpiHeaderError::Parse),
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for UnpiPacket<'a> {
    type Error = UnpiHeaderError;
    fn try_from(data: &'a [u8]) -> Result<UnpiPacket, Self::Error> {
        println!("data[0] = {:?}, data[1] = {:?}", data[0], data[1]);
        let len: u16 = (data[0] as u16) << 5 | data[1] as u16;
        println!("len: {:?}", len);
        Ok(UnpiPacket {
            len,
            type_subsystem: Wrapped(data[2])
                .try_into()
                .map_err(|_| UnpiHeaderError::InvalidTypeSubsystem)?,
            command: data[3]
                .try_into()
                .map_err(|_| UnpiHeaderError::InvalidCommand)?,
            payload: &data[4..(4 + len as usize)],
            fcs: data[data.len() - 1],
        })
    }
}

struct Wrapped<T>(T);

impl Into<Wrapped<u8>> for (MessageType, Subsystem) {
    fn into(self) -> Wrapped<u8> {
        Wrapped(Into::<u8>::into(self.0) << 5 | Into::<u8>::into(self.1))
    }
}

impl TryFrom<Wrapped<u8>> for (MessageType, Subsystem) {
    type Error = UnpiHeaderError;

    fn try_from(value: Wrapped<u8>) -> Result<Self, Self::Error> {
        let v = value.0;
        let message_type = (v & 0b1110_0000) >> 5;
        let subsystem = v & 0b0001_1111;
        Ok((
            message_type.try_into()?,
            subsystem
                .try_into()
                .map_err(|_| UnpiHeaderError::InvalidTypeSubsystem)?,
        ))
    }
}

impl<'a> UnpiPacket<'a> {
    pub fn from_payload(
        payload: &'a [u8],
        type_subsystem: (MessageType, Subsystem),
        command: u8,
    ) -> UnpiPacket<'a> {
        let h = UnpiPacket {
            len: payload.len() as u16 + 3,
            type_subsystem,
            command,
            payload,
            fcs: 0,
        };
        let fcs = h.checksum();
        UnpiPacket { fcs, ..h }
    }

    pub fn to_bytes(&self, output: &mut [u8]) -> u8 {
        output[0..2].copy_from_slice(&self.len.to_le_bytes());
        output[2] = Into::<Wrapped<u8>>::into(self.type_subsystem.clone()).0;
        output[3] = self.command.into();
        output[4..4 + self.payload.len()].copy_from_slice(self.payload);
        output[4 + self.payload.len()] = self.fcs;
        4 + self.payload.len() as u8
    }

    #[allow(unused)]
    //https://github.com/Mr-Markus/unpi-net/blob/master/src/UnpiNet/Packet.cs
    //todo: test
    pub fn checksum_buffer(buf1: &[u8]) -> u8 {
        let mut fcs: u8 = 0x00;

        for &byte in buf1 {
            fcs ^= byte;
        }

        fcs
    }

    pub fn checksum(&self) -> u8 {
        let mut output = [0u8; MAX_FRAME_SIZE];
        let end = self.to_bytes(&mut output) as usize;
        Self::checksum_buffer(&mut output[0..end])
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

    #[test]
    pub fn test_unpi_header() {
        let data = [0xFEu8, 0x00, 0x05, 0x37, 0x04, 0x01, 0x02, 0x03, 0x04, 0x07];
        let header = UnpiPacket::try_from(&data[..]).unwrap();
        let checksum = UnpiPacket::checksum_buffer(&data[1..data.len() - 1]);
        assert_eq!(header.len, 5);
        assert_eq!(header.type_subsystem, (MessageType::SREQ, Subsystem::Res0));
        assert_eq!(header.command, 0x04);
        assert_eq!(header.payload, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(checksum, header.fcs);
        assert_eq!(header.fcs, 0x07);
        assert_eq!(checksum, header.checksum());
    }

    //https://github.com/shimmeringbee/unpi/blob/main/frame_test.go

    #[test]
    pub fn test_unpi_empty() {
        let data = [0xFEu8, 0x00, 0x25, 0x37, 0x12, 0x01];
        let header = UnpiPacket::try_from(&data[1..]).unwrap();
        assert_eq!(header.type_subsystem, (MessageType::SREQ, Subsystem::Zdo));
        assert_eq!(header.command, 0x37);
        assert_eq!(header.payload.len(), 0);
    }
}
