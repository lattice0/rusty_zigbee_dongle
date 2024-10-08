use crate::{
    serial::SerialThreadError,
    utils::map::MapError,
    zstack::{
        nv_memory::nv_item::NvMemoryAdapterError,
        unpi::{
            constants::{CommandStatus, NoCommandStatusError},
            serial::UnpiCommandError,
            subsystems::{sys::VersionResponse, util::GetDeviceInfoResponse},
        },
    },
};
use deku::{reader::Reader, writer::Writer, DekuError, DekuReader, DekuWriter};
use std::{
    future::Future,
    io::{Read, Seek, Write},
};

pub type OnEvent = Box<dyn Fn(ZigbeeEvent) -> Result<(), CoordinatorError> + Send + Sync>;

pub trait Coordinator {
    type ZclFrame;
    type ZclPayload<'a>;
    type IeeAddress;

    fn start(&self) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn stop(&self) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn version(&self) -> impl Future<Output = Result<VersionResponse, CoordinatorError>>;
    fn permit_join(
        &self,
        duration: std::time::Duration,
        address: Option<u16>,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn is_inter_pan_mode(&self) -> impl Future<Output = bool>;
    fn reset(&self, reset_type: ResetType) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn set_led(&self, led_status: LedStatus) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn discover_route(
        &self,
        address: Option<u16>,
        wait: Option<bool>,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn change_channel(&self, channel: u8) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn set_transmit_power(&self, power: i8) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn request_network_address(addr: &str) -> impl Future<Output = Result<(), CoordinatorError>>;
    #[allow(clippy::too_many_arguments)]
    fn send_zcl_frame(
        &self,
        iee_addr: &Self::IeeAddress,
        network_address: u16,
        endpoint: u16,
        zcl_frame: &Self::ZclFrame,
        timeout: std::time::Duration,
        disable_response: bool,
        disable_recovery: bool,
        source_endpoint: Option<u32>,
    ) -> impl Future<Output = Result<Option<Self::ZclPayload<'static>>, CoordinatorError>>;
    fn set_on_event(
        &mut self,
        on_zigbee_event: OnEvent,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn error_if_interpan_mode(&self) -> impl Future<Output = Result<(), CoordinatorError>> {
        async {
            if self.is_inter_pan_mode().await {
                Err(CoordinatorError::InterpanMode)
            } else {
                Ok(())
            }
        }
    }
    fn device_info(&self) -> impl Future<Output = Result<GetDeviceInfoResponse, CoordinatorError>>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct DeviceInfo {
    pub status: u8,
    pub ieee_addr: [u8; 8],
    pub short_addr: u16,
    pub device_type: u8,
    pub device_state: u8,
    pub num_assoc_devices: u8,
    pub assoc_devices_list: [u16; 16],
}

#[derive(Debug, Copy, Clone)]
#[allow(clippy::type_complexity)]
pub enum ZigbeeEvent {
    DeviceJoined {
        network_address: u16,
        ieee_address: [u8; 8],
    },
    DeviceAnnounce {
        network_address: u16,
        ieee_address: [u8; 8],
    },
    NetworkAddress {
        network_address: u16,
        ieee_address: [u8; 8],
    },
    DeviceLeave(Either<(Option<u16>, [u8; 8]), (u16, Option<[u8; 8]>)>),
}

#[derive(Debug, Copy, Clone)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

pub enum AddressMode {
    AddrNotPresent = 0,
    AddrGroup = 1,
    Addr16bit = 2,
    Addr64bit = 3,
    AddrBroadcast = 15,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LedStatus {
    Disable,
    On,
    Off,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ResetType {
    Soft,
    Hard,
}

impl DekuWriter<()> for ResetType {
    #[doc = " Write type to bytes"]
    fn to_writer<W: Write + Seek>(
        &self,
        writer: &mut Writer<W>,
        _ctx: (),
    ) -> Result<(), DekuError> {
        match self {
            ResetType::Soft => writer.write_bytes(&[1u8])?,
            ResetType::Hard => writer.write_bytes(&[0u8])?,
        }
        Ok(())
    }
}

impl<'a> DekuReader<'a, ()> for ResetType {
    fn from_reader_with_ctx<R: Read + Seek>(
        reader: &mut Reader<R>,
        _ctx: (),
    ) -> Result<Self, DekuError>
    where
        Self: Sized,
    {
        let mut output = [0u8; 1];
        reader.read_bytes(1, &mut output)?;
        match output[0] {
            0 => Ok(ResetType::Hard),
            1 => Ok(ResetType::Soft),
            _ => Err(DekuError::Parse("Invalid reset type".into())),
        }
    }
}

#[derive(Debug)]
pub enum CoordinatorError {
    SerialOpen(String),
    Serial(SerialThreadError),
    SerialWrite,
    SerialRead,
    NoCommandWithName(String),
    Io(String),
    InvalidChannel,
    ResponseMismatch,
    Map(MapError),
    NoRequest,
    NoResponse,
    SerialChannelMissing,
    SubscriptionError,
    InterpanMode,
    DurationTooLong,
    CoordinatorOpen,
    MissingKey,
    ParameterNotFound(String),
    InvalidCommandStatus,
    InvalidResponse,
    InvalidMessageType,
    NvMemoryAdapter(NvMemoryAdapterError),
    UnpiCommand(UnpiCommandError),
    CommandStatusFailure(CommandStatus),
    NoCommandStatus(NoCommandStatusError),
    Deku(deku::DekuError),
}

impl From<std::io::Error> for CoordinatorError {
    fn from(e: std::io::Error) -> Self {
        CoordinatorError::Io(e.to_string())
    }
}

impl From<MapError> for CoordinatorError {
    fn from(e: MapError) -> Self {
        CoordinatorError::Map(e)
    }
}

impl From<NvMemoryAdapterError> for CoordinatorError {
    fn from(e: NvMemoryAdapterError) -> Self {
        CoordinatorError::NvMemoryAdapter(e)
    }
}

impl From<UnpiCommandError> for CoordinatorError {
    fn from(e: UnpiCommandError) -> Self {
        CoordinatorError::UnpiCommand(e)
    }
}

impl From<NoCommandStatusError> for CoordinatorError {
    fn from(e: NoCommandStatusError) -> Self {
        CoordinatorError::NoCommandStatus(e)
    }
}

impl From<DekuError> for CoordinatorError {
    fn from(e: DekuError) -> Self {
        CoordinatorError::Deku(e)
    }
}
