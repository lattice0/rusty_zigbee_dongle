use crate::{unpi::commands::ParameterError, utils::map::MapError};
use std::future::Future;

pub trait Coordinator {
    type ZclFrame;
    type ZclPayload<'a>;
    type IeeAddress;

    fn start(&self) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn stop(&self) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn version(&self) -> impl Future<Output = Result<usize, CoordinatorError>>;
    fn permit_join(
        &self,
        address: u16,
        duration: std::time::Duration,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn reset(&self, reset_type: ResetType) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn set_led(
        &self,
        led_status: LedStatus,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn change_channel(&self, channel: u8)
        -> impl Future<Output = Result<(), CoordinatorError>>;
    fn set_transmit_power(
        &self,
        power: i8,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;
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
    SerialOpen,
    SerialWrite,
    NoCommandWithName,
    Io,
    Parameter(ParameterError),
    InvalidChannel,
    RequestMismatch,
    ResponseMismatch,
    Map(MapError)
}

impl From<std::io::Error> for CoordinatorError {
    fn from(_e: std::io::Error) -> Self {
        CoordinatorError::Io
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
