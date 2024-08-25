pub const MAX_FRAME_SIZE: usize = 255;

pub struct Unpi {}

/*************************************************************************************************/
/*** TI Unified NPI Packet Format                                                              ***/
/***     SOF(1) + Length(2/1) + Type/Sub(1) + Cmd(1) + Payload(N) + FCS(1)                     ***/
/*************************************************************************************************/
#[derive(Debug, PartialEq)]
pub struct UnpiHeader<'a> {
    pub len: u16,
    pub type_subsystem: MessageTypes,
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
}

#[derive(Debug, PartialEq, Clone)]
pub enum MessageTypes {
    SREQ,
    AREQ,
    SRESP,
}

impl Into<u8> for MessageTypes {
    fn into(self) -> u8 {
        match self {
            MessageTypes::SREQ => 0,
            MessageTypes::AREQ => 1,
            MessageTypes::SRESP => 2,
        }
    }
}

impl Unpi {
    pub fn new() -> Unpi {
        Unpi {}
    }
}

#[derive(Debug, PartialEq)]
pub enum CommandType {
    POLL,
    SREQ,
    AREQ,
    SRSP,
    RES0,
    RES1,
    RES2,
    RES3,
}

#[derive(Debug, PartialEq)]
pub enum Subsystem {
    RpcSysRes0,
    RpcSysSys,
    RpcSysMac,
    RpcSysNwk,
    RpcSysAf,
    RpcSysZdo,
    RpcSysSapi,
    RpcSysUtil,
    RpcSysDbg,
    RpcSysApp,
    RpcSysRcaf,
    RpcSysRcn,
    RpcSysRcnClient,
    RpcSysBoot,
    RpcSysZiptest,
    RpcSysDebug,
    RpcSysPeripherals,
    RpcSysNfc,
    RpcSysPbNwkMgr,
    RpcSysPbGw,
    RpcSysPbOtaMgr,
    RpcSysBleSpnp,
    RpcSysBleHci,
    RpcSysResv01,
    RpcSysResv02,
    RpcSysResv03,
    RpcSysResv04,
    RpcSysResv05,
    RpcSysResv06,
    RpcSysResv07,
    RpcSysResv08,
    RpcSysSrvCtr,
}

impl TryFrom<u8> for CommandType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CommandType::POLL),
            1 => Ok(CommandType::SREQ),
            2 => Ok(CommandType::AREQ),
            3 => Ok(CommandType::SRSP),
            4 => Ok(CommandType::RES0),
            5 => Ok(CommandType::RES1),
            6 => Ok(CommandType::RES2),
            7 => Ok(CommandType::RES3),
            _ => Err(()),
        }
    }
}

impl Into<u8> for CommandType {
    fn into(self) -> u8 {
        match self {
            CommandType::POLL => 0,
            CommandType::SREQ => 1,
            CommandType::AREQ => 2,
            CommandType::SRSP => 3,
            CommandType::RES0 => 4,
            CommandType::RES1 => 5,
            CommandType::RES2 => 6,
            CommandType::RES3 => 7,
        }
    }
}

impl TryFrom<u8> for MessageTypes {
    type Error = UnpiHeaderError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageTypes::SREQ),
            1 => Ok(MessageTypes::AREQ),
            2 => Ok(MessageTypes::SRESP),
            _ => Err(UnpiHeaderError::Parse),
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for UnpiHeader<'a> {
    type Error = UnpiHeaderError;
    fn try_from(data: &'a [u8]) -> Result<UnpiHeader, Self::Error> {
        Ok(UnpiHeader {
            len: u16::from_le_bytes([data[0], data[1]]),
            type_subsystem: MessageTypes::try_from(data[2])
                .map_err(|_| UnpiHeaderError::InvalidTypeSubsystem)?,
            command: data[3],
            payload: &data[4..data.len() - 1],
            fcs: data[data.len() - 1],
        })
    }
}

impl<'a> UnpiHeader<'a> {
    pub fn to_bytes(&self, output: &mut [u8]) -> u8 {
        output[0..2].copy_from_slice(&self.len.to_le_bytes());
        output[2] = self.type_subsystem.clone().into();
        output[3] = self.command;
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
        match value {
            0 => Ok(Subsystem::RpcSysRes0),
            1 => Ok(Subsystem::RpcSysSys),
            2 => Ok(Subsystem::RpcSysMac),
            3 => Ok(Subsystem::RpcSysNwk),
            4 => Ok(Subsystem::RpcSysAf),
            5 => Ok(Subsystem::RpcSysZdo),
            6 => Ok(Subsystem::RpcSysSapi),
            7 => Ok(Subsystem::RpcSysUtil),
            8 => Ok(Subsystem::RpcSysDbg),
            9 => Ok(Subsystem::RpcSysApp),
            10 => Ok(Subsystem::RpcSysRcaf),
            11 => Ok(Subsystem::RpcSysRcn),
            12 => Ok(Subsystem::RpcSysRcnClient),
            13 => Ok(Subsystem::RpcSysBoot),
            14 => Ok(Subsystem::RpcSysZiptest),
            15 => Ok(Subsystem::RpcSysDebug),
            16 => Ok(Subsystem::RpcSysPeripherals),
            17 => Ok(Subsystem::RpcSysNfc),
            18 => Ok(Subsystem::RpcSysPbNwkMgr),
            19 => Ok(Subsystem::RpcSysPbGw),
            20 => Ok(Subsystem::RpcSysPbOtaMgr),
            21 => Ok(Subsystem::RpcSysBleSpnp),
            22 => Ok(Subsystem::RpcSysBleHci),
            23 => Ok(Subsystem::RpcSysResv01),
            24 => Ok(Subsystem::RpcSysResv02),
            25 => Ok(Subsystem::RpcSysResv03),
            26 => Ok(Subsystem::RpcSysResv04),
            27 => Ok(Subsystem::RpcSysResv05),
            28 => Ok(Subsystem::RpcSysResv06),
            29 => Ok(Subsystem::RpcSysResv07),
            30 => Ok(Subsystem::RpcSysResv08),
            31 => Ok(Subsystem::RpcSysSrvCtr),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_unpi_header() {
        let data = [0xFEu8, 0x00, 0x05, 0x02, 0x04, 0x01, 0x02, 0x03, 0x04, 0x07];
        let header = UnpiHeader::try_from(&data[1..]).unwrap();
        let checksum = UnpiHeader::checksum_buffer(&data[1..data.len() - 1]);
        assert_eq!(header.len, 0x0500);
        assert_eq!(header.type_subsystem, super::MessageTypes::SRESP);
        assert_eq!(header.command, 0x04);
        assert_eq!(header.payload, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(checksum, header.fcs);
        assert_eq!(header.fcs, 0x07);
        assert_eq!(checksum, header.checksum());
    }
}
