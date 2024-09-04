use crate::{
    coordinator::{AddressMode, Coordinator, CoordinatorError, LedStatus, ResetType},
    unpi::{
        commands::{get_command_by_name, ParameterValue, ParametersValueMap},
        LenTypeInfo, MessageType, Subsystem, UnpiPacket, MAX_FRAME_SIZE,
    }, utils::{log, warnn},
};
use futures::lock::Mutex;
use serialport::SerialPort;
use std::{path::PathBuf, sync::Arc, time::Duration};

//TODO: fix this
const MAXIMUM_ZIGBEE_PAYLOAD_SIZE: usize = 255;

pub struct CC2531X {
    _supports_led: Option<bool>,
    serial: Arc<Mutex<Box<dyn SerialPort>>>,
}

impl CC2531X {
    pub fn from_path(path: PathBuf, baud_rate: u32) -> Result<Self, CoordinatorError> {
        let serial = serialport::new(path.to_str().unwrap(), baud_rate)
            .timeout(Duration::from_millis(10))
            .open()
            .map_err(|_e| CoordinatorError::SerialOpen)?;
        Ok(Self {
            serial: Arc::new(Mutex::new(serial)),
            _supports_led: None,
        })
    }

    pub async fn wait_for(
        &self,
        name: &str,
        message_type: MessageType,
        subsystem: Subsystem,
        _timeout: Option<std::time::Duration>,
    ) -> Result<ParametersValueMap, CoordinatorError> {
        let command =
            get_command_by_name(&subsystem, name).ok_or(CoordinatorError::NoCommandWithName)?;
        let mut buffer = [0; MAX_FRAME_SIZE];
        let len = self
            .serial
            .lock()
            .await
            .read(&mut buffer)
            .map_err(|_e| CoordinatorError::Io)?;
        let packet = UnpiPacket::from_payload(
            (&buffer[..len], LenTypeInfo::OneByte),
            (message_type, subsystem),
            command.id,
        )?;
        log!("<<< {:?}", packet);
        if packet.type_subsystem == (message_type, subsystem) && packet.command == command.id {
            let response = command.read_and_fill(packet.payload)?;
            Ok(response)
        } else {
            warnn!("rejecting packet: {:?}", packet);
            Err(CoordinatorError::Io)
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

    async fn version(&self) -> Result<Option<ParameterValue>, CoordinatorError> {
        let command = get_command_by_name(&Subsystem::Sys, "version")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        let mut lock = self.serial.lock().await;
        let send = UnpiPacket::from_command_to_serial_async(
            command.id,
            command,
            &[],
            (MessageType::SREQ, Subsystem::Sys),
            &mut **lock,
        );
        let wait = self.wait_for("version", MessageType::SRESP, Subsystem::Sys, None);
        let r = futures::try_join!(send, wait)?;
        Ok(r.1.get(&"majorrel").cloned())
    }

    async fn reset(&self, reset_type: ResetType) -> Result<(), CoordinatorError> {
        let command = get_command_by_name(&Subsystem::Sys, "reset_req")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        let parameters = match reset_type {
            ResetType::Soft => &[("type", ParameterValue::U8(1))],
            ResetType::Hard => &[("type", ParameterValue::U8(0))],
        };

        let mut lock = self.serial.lock().await;
        UnpiPacket::from_command_to_serial(
            command.id,
            command,
            parameters,
            (MessageType::SREQ, Subsystem::Util),
            &mut **lock,
        )?;

        Ok(())
    }

    async fn set_led(&self, led_status: LedStatus) -> Result<(), CoordinatorError> {
        let command = get_command_by_name(&Subsystem::Util, "led_control")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        //TODO: const firmwareControlsLed = parseInt(this.version.revision) >= 20211029;
        let firmware_controls_led = true;
        let mut lock = self.serial.lock().await;
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
            &mut **lock,
        )?;

        Ok(())
    }

    async fn change_channel(&self, channel: u8) -> Result<(), CoordinatorError> {
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
        let mut lock = self.serial.lock().await;
        UnpiPacket::from_command_to_serial(
            command.id,
            command,
            parameters,
            (MessageType::SREQ, Subsystem::Zdo),
            &mut **lock,
        )?;

        Ok(())
    }

    async fn set_transmit_power(&self, power: i8) -> Result<(), CoordinatorError> {
        let parameters = &[
            ("operation", ParameterValue::U8(0)),
            ("value", ParameterValue::I8(power)),
        ];

        let command = get_command_by_name(&Subsystem::Zdo, "stack_tune")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        let mut lock = self.serial.lock().await;
        UnpiPacket::from_command_to_serial(
            command.id,
            command,
            parameters,
            (MessageType::SREQ, Subsystem::Zdo),
            &mut **lock,
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
