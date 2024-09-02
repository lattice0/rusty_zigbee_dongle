use crate::unpi::commands::ParameterError;
use std::future::Future;

pub trait Coordinator {
    type ZclFrame;
    type ZclPayload<'a>;
    type IeeAddress;

    fn start(&self) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn stop(&self) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn permit_join(
        &self,
        address: u16,
        duration: std::time::Duration,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn reset(&self, reset_type: ResetType) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn set_led(
        &mut self,
        led_status: LedStatus,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn change_channel(&mut self, channel: u8)
        -> impl Future<Output = Result<(), CoordinatorError>>;
    fn set_transmit_power(
        &mut self,
        power: i8,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;
    fn request_network_address(addr: &str) -> impl Future<Output = Result<(), CoordinatorError>>;
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
    SerialWrite(String),
    NoCommandWithName,
    Io(String),
    Parameter(ParameterError),
    InvalidChannel,
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
