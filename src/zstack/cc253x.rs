use super::{
    nv_memory::nv_memory::NvMemoryAdapter,
    unpi::serial::{request, request_with_reply},
};
use crate::{
    coordinator::{
        AddressMode, Coordinator, CoordinatorError, DeviceInfo, LedStatus, OnEvent, ResetType,
        ZigbeeEvent,
    },
    parameters::ParameterValue,
    serial::{simple_serial_port::SimpleSerialPort, SimpleSerial},
    subscription::{Event, Predicate, Subscription, SubscriptionService},
    utils::{error, info, trace, warn},
    zstack::unpi::{
        commands::{Command, ParametersValueMap},
        constants::{af, CommandStatus},
        serial::wait_for,
        MessageType, SUnpiPacket, Subsystem,
    },
};
use futures::{executor::block_on, lock::Mutex};
use std::{ops::Deref, sync::Arc};

//TODO: fix this
const MAXIMUM_ZIGBEE_PAYLOAD_SIZE: usize = 255;

pub struct CC253X<S: SimpleSerial<SUnpiPacket>> {
    _supports_led: Option<bool>,
    // Subscribe to events (packets and others) here
    subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
    // Send data directly to serial here, but for reading we use the subscription service above
    serial: Arc<Mutex<S>>,
    on_zigbee_event: Arc<Mutex<Option<OnEvent>>>,
    pub nv_adapter: NvMemoryAdapter<S>,
}

impl CC253X<SimpleSerialPort<SUnpiPacket>> {
    pub async fn from_simple_serial(path: &str, baud_rate: u32) -> Result<Self, CoordinatorError> {
        let on_zigbee_event = Arc::new(Mutex::new(Option::<OnEvent>::None));
        let subscriptions = Arc::new(Mutex::new(SubscriptionService::new()));
        let serial = SimpleSerialPort::new(path, baud_rate, subscriptions.clone())?;
        let device_announce_command = get_command_by_name(&Subsystem::Zdo, "tc_device_index")
            .ok_or(CoordinatorError::NoCommandWithName(
                "tc_device_index".to_string(),
            ))?;

        let on_zigbee_event_clone = on_zigbee_event.clone();
        subscriptions.lock().await.subscribe(Subscription::Event(
            Predicate(Box::new(|packet: &SUnpiPacket| {
                packet.type_subsystem == (MessageType::AREQ, Subsystem::Zdo)
                    && packet.command == device_announce_command.id
            })),
            Event(Box::new(move |packet: &SUnpiPacket| {
                let a = async {
                    if let Some(on_zigbee_event) = on_zigbee_event_clone.lock().await.deref() {
                        (on_zigbee_event)(ZigbeeEvent::DeviceAnnounce {
                            network_address: packet.payload[0] as u16,
                            ieee_address: packet.payload[1..9].try_into().unwrap(),
                        })
                        .unwrap();
                    }
                };
                block_on(a);
            })),
        ));

        let serial = Arc::new(Mutex::new(serial));
        Ok(Self {
            serial: serial.clone(),
            _supports_led: None,
            subscriptions: subscriptions.clone(),
            on_zigbee_event,
            nv_adapter: NvMemoryAdapter::new(serial, subscriptions)?,
        })
    }
}

impl<S: SimpleSerial<SUnpiPacket>> CC253X<S> {
    // helper proxy function
    pub async fn request(
        &self,
        name: &str,
        subsystem: Subsystem,
        parameters: &[(&'static str, ParameterValue)],
    ) -> Result<(), CoordinatorError> {
        Ok(request(name, subsystem, parameters, self.serial.clone()).await?)
    }

    // helper proxy function
    async fn wait_for(
        &self,
        name: &str,
        message_type: MessageType,
        subsystem: Subsystem,
        timeout: Option<std::time::Duration>,
    ) -> Result<(SUnpiPacket, Command), CoordinatorError> {
        Ok(wait_for(
            name,
            message_type,
            subsystem,
            self.subscriptions.clone(),
            timeout,
        )
        .await?)
    }

    // helper proxy function
    pub async fn request_with_reply(
        &self,
        name: &str,
        subsystem: Subsystem,
        parameters: &[(&'static str, ParameterValue)],
        timeout: Option<std::time::Duration>,
    ) -> Result<ParametersValueMap, CoordinatorError> {
        Ok(request_with_reply(
            name,
            subsystem,
            parameters,
            self.serial.clone(),
            self.subscriptions.clone(),
            timeout,
        )
        .await?)
    }

    pub async fn begin_startup(&self) -> Result<CommandStatus, CoordinatorError> {
        info!("beginning startup...");
        let command_name = "state_changed_ind";
        let command = get_command_by_name(&Subsystem::Zdo, command_name).ok_or(
            CoordinatorError::NoCommandWithName(command_name.to_string()),
        )?;
        let wait = self.wait_for(&command.name, MessageType::AREQ, Subsystem::Zdo, None);
        let send = self.request(
            "startup_from_app",
            Subsystem::Zdo,
            &[("start_delay", ParameterValue::U16(100))],
        );
        let r = futures::try_join!(send, wait).map(|(_, (packet, command))| {
            info!("reading filling command: {:?}", command);
            command.read_and_fill(packet.payload.as_slice())
        })??;
        let c = TryInto::<CommandStatus>::try_into(r.clone());
        let r = match c {
            Ok(c) => c,
            Err(_) => {
                error!("error converting to CommandStatus: {:?}", r);
                return Err(CoordinatorError::InvalidCommandStatus);
            }
        };
        Ok(r)
    }
}

impl<S: SimpleSerial<SUnpiPacket>> Coordinator for CC253X<S> {
    type ZclFrame = psila_data::cluster_library::ClusterLibraryHeader;

    type ZclPayload<'a> = ([u8; MAXIMUM_ZIGBEE_PAYLOAD_SIZE], usize);

    type IeeAddress = ieee802154::mac::Address;

    async fn start(&self) -> Result<(), CoordinatorError> {
        for attempt in 0..3 {
            trace!("pinging coordinator attempt number {:?}", attempt);
            let ping = self
                .request_with_reply("ping", Subsystem::Sys, &[], None)
                .await?;
            if ping.get(&"capabilities").is_some() {
                trace!("ping successful");
                let version = self.version().await?;
                if let Some(version) = version {
                    info!("coordinator version: {:?}", version);
                    return Ok(());
                } else {
                    error!("no version found");
                    Err(CoordinatorError::CoordinatorOpen)?;
                }
            } else {
                error!("ping failed");
            }
        }
        Err(CoordinatorError::CoordinatorOpen)
    }

    async fn stop(&self) -> Result<(), CoordinatorError> {
        Ok(())
    }

    async fn is_inter_pan_mode(&self) -> bool {
        warn!("is_inter_pan_mode not implemented, assuming false");
        false
    }

    async fn version(&self) -> Result<Option<ParameterValue>, CoordinatorError> {
        let version = self
            .request_with_reply("version", Subsystem::Sys, &[], None)
            .await?;
        Ok(version.get(&"majorrel").cloned())
    }

    async fn reset(&self, reset_type: ResetType) -> Result<(), CoordinatorError> {
        trace!("reset with reset type {:?}", reset_type);
        let parameters = match reset_type {
            ResetType::Soft => &[("type", ParameterValue::U8(1))],
            ResetType::Hard => &[("type", ParameterValue::U8(0))],
        };
        self.request("reset_req", Subsystem::Sys, parameters)
            .await?;
        Ok::<(), CoordinatorError>(())
    }

    async fn set_led(&self, led_status: LedStatus) -> Result<(), CoordinatorError> {
        trace!("setting LED to {:?}", led_status);
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
        info!("changing channel to {}", channel);
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
        info!("setting transmit power to {}", power);
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
        info!("permitting join for {} seconds", seconds.as_secs());
        self.error_if_interpan_mode().await?;
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
        address: Option<u16>,
        wait: Option<bool>,
    ) -> Result<(), CoordinatorError> {
        trace!(
            "discovering route at address: {:?}, wait: {:?}",
            address,
            wait
        );
        let parameters = &[
            ("destination_address", ParameterValue::U16(0)),
            ("options", ParameterValue::U8(0)),
            ("radius", ParameterValue::U8(af::DEFAULT_RADIUS)),
        ];

        self.request("exit_route_disc", Subsystem::Zdo, parameters)
            .await
    }

    async fn set_on_event(&mut self, on_zigbee_event: OnEvent) -> Result<(), CoordinatorError> {
        self.on_zigbee_event
            .lock()
            .await
            .replace(Box::new(on_zigbee_event));
        Ok(())
    }

    async fn device_info(&self) -> Result<DeviceInfo, CoordinatorError> {
        info!("getting device info...");
        let device_info: DeviceInfo = self
            .request_with_reply("get_device_info", Subsystem::Util, &[], None)
            .await?
            .try_into()?;
        Ok(device_info)
    }
}
