use crate::{
    coordinator::{AddressMode, Coordinator, CoordinatorError, LedStatus, ResetType},
    serial::{simple_serial_port::SimpleSerialPort, SubscriptionSerial},
    subscription::{Predicate, Subscription, SubscriptionService},
    unpi::{
        commands::{get_command_by_name, ParameterValue, ParametersValueMap},
        LenTypeInfo, MessageType, SUnpiPacket, Subsystem,
    },
    utils::warnn,
};
use futures::{
    channel::oneshot::{self, Receiver, Sender},
    lock::Mutex,
};
use std::{path::PathBuf, sync::Arc};

//TODO: fix this
const MAXIMUM_ZIGBEE_PAYLOAD_SIZE: usize = 255;

pub struct CC253X<S: SubscriptionSerial> {
    _supports_led: Option<bool>,
    subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
    serial: Arc<Mutex<S>>,
    on_zcl_frame_callback: Option<Box<dyn Fn() -> Result<(), CoordinatorError>>>,
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
            on_zcl_frame_callback: None,
        })
    }
}

impl<S: SubscriptionSerial> CC253X<S> {
    pub async fn request(
        &self,
        name: &str,
        subsystem: Subsystem,
        parameters: &[(&'static str, ParameterValue)],
    ) -> Result<(), CoordinatorError> {
        let command =
            get_command_by_name(&subsystem, name).ok_or(CoordinatorError::NoCommandWithName)?;
        let packet = SUnpiPacket::from_command_owned(
            LenTypeInfo::OneByte,
            (command.command_type, subsystem),
            parameters,
            command,
        )?;
        self.serial.lock().await.write(&packet).await?;
        Ok(())
    }

    pub async fn request_with_reply(
        &self,
        name: &str,
        subsystem: Subsystem,
        parameters: &[(&'static str, ParameterValue)],
    ) -> Result<ParametersValueMap, CoordinatorError> {
        let command =
            get_command_by_name(&subsystem, name).ok_or(CoordinatorError::NoCommandWithName)?;
        let packet = SUnpiPacket::from_command_owned(
            LenTypeInfo::OneByte,
            (command.command_type, subsystem),
            parameters,
            command,
        )?;
        let wait = self.wait_for("version", MessageType::SRESP, Subsystem::Sys, None);
        let send = async {
            let mut lock = self.serial.lock().await;
            lock.write(&packet).await
        };
        futures::try_join!(send, wait)
            .map(|(_, packet)| command.read_and_fill(packet.payload.as_slice()))?
    }

    pub async fn wait_for(
        &self,
        name: &str,
        message_type: MessageType,
        subsystem: Subsystem,
        _timeout: Option<std::time::Duration>,
    ) -> Result<SUnpiPacket, CoordinatorError> {
        let command =
            get_command_by_name(&subsystem, name).ok_or(CoordinatorError::NoCommandWithName)?;
        let subscriptions = self.subscriptions.clone();
        let (tx, rx): (Sender<SUnpiPacket>, Receiver<SUnpiPacket>) = oneshot::channel();
        {
            let mut s = subscriptions.lock().await;
            let subscription = Subscription::SingleShot(
                Predicate(Box::new(move |packet: &SUnpiPacket| {
                    packet.type_subsystem == (message_type, subsystem)
                        && packet.command == command.id
                })),
                tx,
            );
            s.subscribe(subscription);
        }

        rx.await.map_err(|_| CoordinatorError::SubscriptionError)
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

    async fn is_inter_pan_mode(&self) -> bool {
        warnn!("is_inter_pan_mode not implemented, assuming false");
        false
    }

    async fn version(&self) -> Result<Option<ParameterValue>, CoordinatorError> {
        let version = self
            .request_with_reply("version", Subsystem::Sys, &[])
            .await?;
        Ok(version.get(&"majorrel").cloned())
    }

    async fn reset(&self, reset_type: ResetType) -> Result<(), CoordinatorError> {
        let parameters = match reset_type {
            ResetType::Soft => &[("type", ParameterValue::U8(1))],
            ResetType::Hard => &[("type", ParameterValue::U8(0))],
        };
        self.request("reset_req", Subsystem::Sys, parameters)
            .await?;
        Ok::<(), CoordinatorError>(())
    }

    async fn set_led(&self, led_status: LedStatus) -> Result<(), CoordinatorError> {
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

        self.request("led_control", Subsystem::Util, parameters)
            .await
    }

    async fn change_channel(&self, channel: u8) -> Result<(), CoordinatorError> {
        let parameters = &[
            ("destination_address", ParameterValue::U16(0xffff)),
            (
                "destination_address_mode",
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
            ("network_manager_address", ParameterValue::U16(0)),
        ];

        self.request(
            "management_network_update_request",
            Subsystem::Zdo,
            parameters,
        )
        .await
    }

    async fn set_transmit_power(&self, power: i8) -> Result<(), CoordinatorError> {
        let parameters = &[
            ("operation", ParameterValue::U8(0)),
            ("value", ParameterValue::I8(power)),
        ];

        self.request("stack_tune", Subsystem::Zdo, parameters).await
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

    async fn permit_join(
        &self,
        seconds: std::time::Duration,
        network_address: Option<u16>,
    ) -> Result<(), CoordinatorError> {
        if self.is_inter_pan_mode().await {
            return Err(CoordinatorError::InterpanMode);
        }
        let address_mode =
            network_address.map_or(AddressMode::AddrBroadcast, |_| AddressMode::Addr16bit);
        let destination_address = network_address.unwrap_or(0xfffc);
        let parameters = &[
            ("address_mode", ParameterValue::U16(address_mode as u16)),
            (
                "destination_address",
                ParameterValue::U16(destination_address),
            ),
            (
                "duration",
                ParameterValue::U8(
                    seconds
                        .as_secs()
                        .try_into()
                        .map_err(|_| CoordinatorError::DurationTooLong)?,
                ),
            ),
            ("tc_significance", ParameterValue::U8(0)),
        ];
        self.request("management_permit_join_request", Subsystem::Zdo, parameters)
            .await
    }

    async fn discover_route(
        &self,
        _address: Option<u16>,
        _wait: Option<bool>,
    ) -> Result<(), CoordinatorError> {
        let parameters = &[
            ("destination_address", ParameterValue::U16(0)),
            ("options", ParameterValue::U8(0)),
            (
                "radius",
                ParameterValue::U8(crate::unpi::constants::af::DEFAULT_RADIUS),
            ),
        ];

        self.request("exit_route_disc", Subsystem::Zdo, parameters)
            .await
    }

    async fn set_on_zcl_frame_callback(
        &mut self,
        on_zcl_frame: Box<dyn Fn() -> Result<(), CoordinatorError>>,
    ) -> Result<(), CoordinatorError> {
        self.on_zcl_frame_callback = Some(on_zcl_frame);
        Ok(())
    }
}
