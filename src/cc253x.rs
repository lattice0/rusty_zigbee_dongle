use crate::{
    coordinator::{AddressMode, Coordinator, CoordinatorError, LedStatus, ResetType},
    serial::{simple_serial_port::SimpleSerialPort, SubscriptionSerial},
    subscription::{Predicate, Subscription, SubscriptionService},
    unpi::{
        commands::{get_command_by_name, ParameterValue},
        LenTypeInfo, MessageType, Subsystem, UnpiPacket,
    },
};
use futures::{
    channel::oneshot::{self, Receiver, Sender},
    lock::Mutex,
};
use std::{path::PathBuf, sync::Arc};

//TODO: fix this
const MAXIMUM_ZIGBEE_PAYLOAD_SIZE: usize = 255;

type Container = Vec<u8>;

pub struct CC253X<S: SubscriptionSerial> {
    _supports_led: Option<bool>,
    subscriptions: Arc<Mutex<SubscriptionService<UnpiPacket<Container>>>>,
    serial: Arc<Mutex<S>>,
}

impl CC253X<SimpleSerialPort> {
    pub fn from_path(path: PathBuf, baud_rate: u32) -> Result<Self, CoordinatorError> {
        let subscriptions = Arc::new(Mutex::new(SubscriptionService::new()));

        let mut serial = SimpleSerialPort::new(
            path.to_str()
                .ok_or(CoordinatorError::Io("not a path".to_string()))?,
            baud_rate,
            subscriptions.clone(),
        )?;
        serial.start()?;
        Ok(Self {
            serial: Arc::new(Mutex::new(serial)),
            _supports_led: None,
            subscriptions: subscriptions.clone(),
        })
    }
}

impl<S: SubscriptionSerial> CC253X<S> {
    pub async fn wait_for(
        &self,
        name: &str,
        message_type: MessageType,
        subsystem: Subsystem,
        _timeout: Option<std::time::Duration>,
    ) -> Result<UnpiPacket<Container>, CoordinatorError> {
        let command =
            get_command_by_name(&subsystem, name).ok_or(CoordinatorError::NoCommandWithName)?;
        let subscriptions = self.subscriptions.clone();
        let (tx, rx): (
            Sender<UnpiPacket<Container>>,
            Receiver<UnpiPacket<Container>>,
        ) = oneshot::channel();
        {
            let mut s = subscriptions.lock().await;
            let subscription = Subscription::SingleShot(
                Predicate(Box::new(move |packet: &UnpiPacket<Container>| {
                    packet.type_subsystem == (message_type, subsystem)
                        && packet.command == command.id
                })),
                tx,
            );
            s.subscribe(subscription);
        }

        let packet = rx.await.map_err(|_| CoordinatorError::SubscriptionError)?;
        Ok(packet)
    }
}

impl<S: SubscriptionSerial> Coordinator for CC253X<S> {
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
        let serial = self.serial.clone();
        let wait = self.wait_for("version", MessageType::SRESP, Subsystem::Sys, None);
        let send = async {
            let packet = UnpiPacket::from_command_owned(
                LenTypeInfo::OneByte,
                (MessageType::SREQ, Subsystem::Sys),
                &[],
                command,
            )?;
            serial.lock().await.write(&packet).await?;
            Ok::<(), CoordinatorError>(())
        };
        let (_s, packet) = futures::try_join!(send, wait)?;
        let r = command.read_and_fill(packet.payload.as_slice())?;
        Ok(r.get(&"majorrel").cloned())
    }

    async fn reset(&self, reset_type: ResetType) -> Result<(), CoordinatorError> {
        let command = get_command_by_name(&Subsystem::Sys, "reset_req")
            .ok_or(CoordinatorError::NoCommandWithName)?;
        let parameters = match reset_type {
            ResetType::Soft => &[("type", ParameterValue::U8(1))],
            ResetType::Hard => &[("type", ParameterValue::U8(0))],
        };

        let serial = self.serial.clone();
        let packet = UnpiPacket::from_command_owned(
            LenTypeInfo::OneByte,
            (MessageType::SREQ, Subsystem::Sys),
            parameters,
            command,
        )?;
        serial.lock().await.write(&packet).await?;
        Ok::<(), CoordinatorError>(())
    }

    async fn set_led(&self, led_status: LedStatus) -> Result<(), CoordinatorError> {
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

        let serial = self.serial.clone();
        let packet = UnpiPacket::from_command_owned(
            LenTypeInfo::OneByte,
            (MessageType::SREQ, Subsystem::Util),
            parameters,
            command,
        )?;
        serial.lock().await.write(&packet).await?;
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

        let serial = self.serial.clone();
        let packet = UnpiPacket::from_command_owned(
            LenTypeInfo::OneByte,
            (MessageType::SREQ, Subsystem::Zdo),
            parameters,
            command,
        )?;
        serial.lock().await.write(&packet).await?;

        Ok(())
    }

    async fn set_transmit_power(&self, power: i8) -> Result<(), CoordinatorError> {
        let parameters = &[
            ("operation", ParameterValue::U8(0)),
            ("value", ParameterValue::I8(power)),
        ];

        let command = get_command_by_name(&Subsystem::Zdo, "stack_tune")
            .ok_or(CoordinatorError::NoCommandWithName)?;

        let serial = self.serial.clone();
        let packet = UnpiPacket::from_command_owned(
            LenTypeInfo::OneByte,
            (MessageType::SREQ, Subsystem::Zdo),
            parameters,
            command,
        )?;
        serial.lock().await.write(&packet).await?;
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
