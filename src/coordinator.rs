use crate::{
    unpi::{commands::ParameterError, parameters::ParameterValue},
    utils::map::{MapError, StaticMap},
};
use std::future::Future;

pub type OnEvent = Box<dyn Fn(ZigbeeEvent) -> Result<(), CoordinatorError>>;

pub trait Coordinator {
    type ZclFrame;
    type ZclPayload<'a>;
    type IeeAddress;

    fn start(&self) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn stop(&self) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn version(&self) -> impl Future<Output = Result<Option<ParameterValue>, CoordinatorError>>;
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
        on_zigbee_event: Box<dyn Fn(ZigbeeEvent) -> Result<(), CoordinatorError>>,
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
    fn device_info(&self) -> impl Future<Output = Result<DeviceInfo, CoordinatorError>>;
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

impl TryFrom<StaticMap<15, &'static str, ParameterValue>> for DeviceInfo {
    type Error = CoordinatorError;

    fn try_from(map: StaticMap<15, &'static str, ParameterValue>) -> Result<Self, Self::Error> {
        Ok(DeviceInfo {
            status: map
                .get(&"status")
                .ok_or(CoordinatorError::MissingKey)?
                .try_into_u8()?,
            ieee_addr: map
                .get(&"ieee_addr")
                .ok_or(CoordinatorError::MissingKey)?
                .try_into_ieee_addr()?,
            short_addr: map
                .get(&"short_addr")
                .ok_or(CoordinatorError::MissingKey)?
                .try_into_u16()?,
            device_type: map
                .get(&"device_type")
                .ok_or(CoordinatorError::MissingKey)?
                .try_into_u8()?,
            device_state: map
                .get(&"device_state")
                .ok_or(CoordinatorError::MissingKey)?
                .try_into_u8()?,
            num_assoc_devices: map
                .get(&"num_assoc_devices")
                .ok_or(CoordinatorError::MissingKey)?
                .try_into_u8()?,
            assoc_devices_list: map
                .get(&"assoc_devices_list")
                .ok_or(CoordinatorError::MissingKey)?
                .try_into_list_u16()?,
        })
    }
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub enum LedStatus {
    Disable,
    On,
    Off,
}

#[derive(Debug, Copy, Clone)]
pub enum ResetType {
    Soft,
    Hard,
}

#[derive(Debug)]
pub enum CoordinatorError {
    SerialOpen(String),
    SerialWrite,
    SerialRead,
    NoCommandWithName(String),
    Io(String),
    Parameter(ParameterError),
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
    InvalidCommandStatus
}

impl From<std::io::Error> for CoordinatorError {
    fn from(e: std::io::Error) -> Self {
        CoordinatorError::Io(e.to_string())
    }
}

impl From<ParameterError> for CoordinatorError {
    fn from(e: ParameterError) -> Self {
        CoordinatorError::Parameter(e)
    }
}

impl From<MapError> for CoordinatorError {
    fn from(e: MapError) -> Self {
        CoordinatorError::Map(e)
    }
}
