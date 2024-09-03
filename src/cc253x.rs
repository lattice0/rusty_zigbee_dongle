use crate::{
    coordinator::{Coordinator, CoordinatorError, LedStatus, ResetType},
    unpi::{
        commands::{get_command_by_name, ParameterValue},
        LenTypeInfo, MessageType, Subsystem, UnpiPacket, MAX_FRAME_SIZE,
    },
    AddressMode,
};
use serialport::SerialPort;
use std::{path::PathBuf, time::Duration};

//TODO: fix this
const MAXIMUM_ZIGBEE_PAYLOAD_SIZE: usize = 255;

pub struct CC2531X {
    _supports_led: Option<bool>,
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
            _supports_led: None,
        })
    }

    pub async fn wait_for(
        &mut self,
        message_type: MessageType,
        subsystem: Subsystem,
        command: u8,
        _timeout: Option<std::time::Duration>,
    ) -> Result<(), CoordinatorError> {
        let mut buffer = [0; MAX_FRAME_SIZE];
        let len = self
            .serial
            .read(&mut buffer)
            .map_err(|e| CoordinatorError::Io(e.to_string()))?;
        let packet = UnpiPacket::from_payload(
            (&buffer[..len], LenTypeInfo::OneByte),
            (message_type, subsystem),
            command,
        )?;
        if packet.type_subsystem == (message_type, subsystem) && packet.command == command {
            Ok(())
        } else {
            Err(CoordinatorError::Io("Unexpected message".to_string()))
        }
    }
}

impl Coordinator for CC2531X {
    type ZclFrame = psila_data::cluster_library::ClusterLibraryHeader;

    type ZclPayload<'a> = ([u8; MAXIMUM_ZIGBEE_PAYLOAD_SIZE], usize);

    type IeeAddress = ieee802154::mac::Address;

    async fn start(&self) -> Result<(), CoordinatorError> {
        todo!()
    }

    async fn stop(&self) -> Result<(), CoordinatorError> {
        todo!()
    }

    async fn permit_join(
        &self,
        _address: u16,
        _duration: std::time::Duration,
    ) -> Result<(), CoordinatorError> {
        todo!()
    }

    async fn reset(&mut self, reset_type: ResetType) -> Result<(), CoordinatorError> {
        let command = get_command_by_name(&Subsystem::Sys, "reset_req")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        let parameters = match reset_type {
            ResetType::Soft => &[("type", ParameterValue::U8(1))],
            ResetType::Hard => &[("type", ParameterValue::U8(0))],
        };

        UnpiPacket::from_command_to_serial(
            command.id,
            command,
            parameters,
            (MessageType::SREQ, Subsystem::Util),
            &mut *self.serial,
        )?;

        Ok(())
    }

    async fn set_led(&mut self, led_status: LedStatus) -> Result<(), CoordinatorError> {
        let command = get_command_by_name(&Subsystem::Util, "led_control")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        //TODO: const firmwareControlsLed = parseInt(this.version.revision) >= 20211029;
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

        UnpiPacket::from_command_to_serial(
            command.id,
            command,
            parameters,
            (MessageType::SREQ, Subsystem::Util),
            &mut *self.serial,
        )?;

        Ok(())
    }

    async fn change_channel(&mut self, channel: u8) -> Result<(), CoordinatorError> {
        let parameters = &[
            ("dst_addr", ParameterValue::U16(0xffff)),
            (
                "dst_addr_mode",
                ParameterValue::U16(AddressMode::AddrBroadcast as u16),
            ),
            (
                "channel_mask",
                ParameterValue::U32(
                    [channel]
                        .into_iter()
                        .reduce(|a, c| a + (1 << c))
                        .ok_or(CoordinatorError::InvalidChannel)? as u32, //TODO: very likely wrong
                ),
            ),
            ("scan_duration", ParameterValue::U8(0xfe)),
            ("scan_count", ParameterValue::U8(0)),
            ("nwk_manager_addr", ParameterValue::U16(0)),
        ];

        let command = get_command_by_name(&Subsystem::Zdo, "management_network_update_request")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        UnpiPacket::from_command_to_serial(
            command.id,
            command,
            parameters,
            (MessageType::SREQ, Subsystem::Zdo),
            &mut *self.serial,
        )?;

        Ok(())
    }

    async fn set_transmit_power(&mut self, power: i8) -> Result<(), CoordinatorError> {
        let parameters = &[
            ("operation", ParameterValue::U8(0)),
            ("value", ParameterValue::I8(power)),
        ];

        let command = get_command_by_name(&Subsystem::Zdo, "stack_tune")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        UnpiPacket::from_command_to_serial(
            command.id,
            command,
            parameters,
            (MessageType::SREQ, Subsystem::Zdo),
            &mut *self.serial,
        )?;
        Ok(())
    }

    async fn request_network_address(_addr: &str) -> Result<(), CoordinatorError> {
        todo!()
    }

    async fn send_zcl_frame(
        &self,
        _iee_addr: &Self::IeeAddress,
        _network_address: u16,
        _endpoint: u16,
        _zcl_frame: &Self::ZclFrame,
        _timeout: std::time::Duration,
        _disable_response: bool,
        _disable_recovery: bool,
        _source_endpoint: Option<u32>,
    ) -> Result<Option<Self::ZclPayload<'static>>, CoordinatorError> {
        Ok(None)
    }
}
