pub trait Coordinator {
    type ZclFrame;
    type ZclPayload<'a>;
    type IeeAddress;

    fn start(&self) -> Result<(), CoordinatorError>;
    fn stop(&self) -> Result<(), CoordinatorError>;
    fn permit_join(
        &self,
        address: u16,
        duration: std::time::Duration,
    ) -> Result<(), CoordinatorError>;
    fn reset(&self, reset_type: ResetType) -> Result<(), CoordinatorError>;
    fn set_led(&self, led_status: LedStatus) -> Result<(), CoordinatorError>;
    fn request_network_address(addr: &str) -> Result<(), CoordinatorError>;
    fn send_zcl_frame<'a>(
        &self,
        iee_addr: &Self::IeeAddress,
        network_address: u16,
        endpoint: u16,
        zcl_frame: &Self::ZclFrame,
        timeout: std::time::Duration,
        disable_response: bool,
        disable_recovery: bool,
        source_endpoint: Option<u32>
    ) -> Result<Option<Self::ZclPayload<'static>>, CoordinatorError>;
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
}
