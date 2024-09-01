use crate::{
    coordinator::{Coordinator, CoordinatorError, LedStatus, ResetType},
    unpi::{
        commands::{get_command_by_name, ParameterValue},
        LenType, LenTypeInfo, MessageType, Subsystem, UnpiPacket,
    },
};
use serialport::SerialPort;
use std::{path::PathBuf, time::Duration};

const MAXIMUM_ZIGBEE_PAYLOAD_SIZE: usize = 255;

pub struct CC2531X {
    supports_led: Option<bool>,
    serial: Box<dyn SerialPort>,
}

impl CC2531X {
    pub fn from_path(path: PathBuf, baud_rate: u32) -> Result<Self, CoordinatorError> {
        let serial = serialport::new(path.to_str().unwrap(), baud_rate)
            .timeout(Duration::from_millis(10))
            .open()
            .map_err(|e| CoordinatorError::SerialOpen(e.to_string()))?;
        Ok(Self {
            serial,
            supports_led: None,
        })
    }
}

#[cfg(feature = "psila")]
impl Coordinator for CC2531X {
    type ZclFrame = psila_data::cluster_library::ClusterLibraryHeader;

    type ZclPayload<'a> = ([u8; MAXIMUM_ZIGBEE_PAYLOAD_SIZE], usize);

    type IeeAddress = ieee802154::mac::Address;

    fn start(&self) -> Result<(), CoordinatorError> {
        todo!()
    }

    fn stop(&self) -> Result<(), CoordinatorError> {
        todo!()
    }

    fn permit_join(
        &self,
        address: u16,
        duration: std::time::Duration,
    ) -> Result<(), CoordinatorError> {
        todo!()
    }

    fn reset(&self, reset_type: ResetType) -> Result<(), CoordinatorError> {
        todo!()
    }

    fn set_led(&mut self, led_status: LedStatus) -> Result<(), CoordinatorError> {
        let command = get_command_by_name(&Subsystem::Util, "led_control")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        //TODO:
        //const firmwareControlsLed = parseInt(this.version.revision) >= 20211029;
        let firmware_controls_led = true;
        let parameters = match led_status {
            LedStatus::Disable => {
                if firmware_controls_led {
                    &[
                        ("led_id", ParameterValue::U8(0xff)),
                        ("mode", ParameterValue::U8(0)),
                    ]
                } else {
                    &[
                        ("led_id", ParameterValue::U8(3)),
                        ("mode", ParameterValue::U8(0)),
                    ]
                }
            }
            LedStatus::On => &[
                ("led_id", ParameterValue::U8(3)),
                ("mode", ParameterValue::U8(1)),
            ],
            LedStatus::Off => &[
                ("led_id", ParameterValue::U8(3)),
                ("mode", ParameterValue::U8(0)),
            ],
        };
        let mut payload_buffer = [0u8; 4];
        let written = command.fill_and_write(parameters, &mut payload_buffer)?;
        let payload: &[u8] = &payload_buffer[0..written];

        let mut unpi_packet_buffer = [0u8; 256];
        let unpi_packet = UnpiPacket::from_payload(
            (payload, LenTypeInfo::OneByte),
            (MessageType::SREQ, Subsystem::Util),
            command.id,
        )?;
        let written = unpi_packet.to_bytes(&mut unpi_packet_buffer)?;
        self.serial
            .write_all(&unpi_packet_buffer[0..written])
            .map_err(|e| CoordinatorError::SerialWrite(e.to_string()))?;
        Ok(())
    }

    fn request_network_address(addr: &str) -> Result<(), CoordinatorError> {
        todo!()
    }

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
    ) -> Result<Option<Self::ZclPayload<'static>>, CoordinatorError> {
        let payload: &[u8] = todo!();
        let unpi_header = UnpiPacket::from_payload(
            (payload, LenTypeInfo::OneByte),
            (MessageType::SREQ, Subsystem::Af),
            0x00,
        );
        let buffer: &[u8] = todo!();
        self.serial
            .write_all(buffer)
            .map_err(|e| CoordinatorError::SerialWrite(e.to_string()))?;
        Ok(None)
    }
}
