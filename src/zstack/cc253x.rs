use super::{
    nv_memory::nv_memory::NvMemoryAdapter,
    unpi::{
        commands::{CommandRequest, CommandResponse},
        serial::{request, request_with_reply},
        subsystems::{
            sys::{VersionRequest, VersionResponse},
            zdo::TcDeviceIndexRequest,
        },
    },
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
        subsystems::{
            sys::{PingRequest, PingResponse, ResetRequest, StackTuneRequest},
            util::{GetDeviceInfoRequest, GetDeviceInfoResponse, LedControlRequest},
            zdo::{
                ExitRouteDiscRequest, ManagementNetworkUpdateRequest, ManagementPermitJoinRequest,
                StartupFromAppRequest, StartupFromAppResponse, StateChangedIndRequest,
            },
        },
        MessageType, SUnpiPacket, Subsystem,
    },
};
use futures::{executor::block_on, lock::Mutex};
use serde::{Deserialize, Serialize};
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

        let on_zigbee_event_clone = on_zigbee_event.clone();
        subscriptions.lock().await.subscribe(Subscription::Event(
            Predicate(Box::new(|packet: &SUnpiPacket| {
                packet.type_subsystem == (MessageType::AREQ, Subsystem::Zdo)
                    && packet.command == TcDeviceIndexRequest::id()
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
    pub async fn request<R: CommandRequest + Serialize>(
        &self,
        command: &R,
    ) -> Result<(), CoordinatorError> {
        let packet = SUnpiPacket::from_command_owned(super::unpi::LenTypeInfo::OneByte, command)?;
        Ok(request::<R, S>(&packet, self.serial.clone()).await?)
    }

    // helper proxy function
    async fn wait_for(
        &self,
        command_id: u8,
        message_type: MessageType,
        subsystem: Subsystem,
        timeout: Option<std::time::Duration>,
    ) -> Result<SUnpiPacket, CoordinatorError> {
        Ok(wait_for(
            command_id,
            message_type,
            subsystem,
            self.subscriptions.clone(),
            timeout,
        )
        .await?)
    }

    // helper proxy function
    pub async fn request_with_reply<
        R: CommandRequest + Serialize,
        Res: CommandResponse + for<'de> Deserialize<'de>,
    >(
        &self,
        command: &R,
        timeout: Option<std::time::Duration>,
    ) -> Result<Res, CoordinatorError> {
        Ok(request_with_reply::<R, S, Res>(
            &SUnpiPacket::from_command_owned(super::unpi::LenTypeInfo::OneByte, command)?,
            self.serial.clone(),
            self.subscriptions.clone(),
            timeout,
        )
        .await?)
    }

    pub async fn begin_startup(&self) -> Result<CommandStatus, CoordinatorError> {
        info!("beginning startup...");
        let command = StateChangedIndRequest { state: 0 };

        let wait = self.wait_for(command.self_id(), MessageType::AREQ, Subsystem::Zdo, None);
        let send = self.request(&StartupFromAppRequest {
            start_delay: 100,
            status: 0,
        });
        let r: StartupFromAppResponse = futures::try_join!(send, wait).map(|(_, packet)| {
            info!("reading filling command: {:?}", command);
            //command.read_and_fill(packet.payload.as_slice());
            packet.to_command_response()
        })??;
        let c = TryInto::<CommandStatus>::try_into(r.clone())?;
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
            let _ping: PingResponse = self.request_with_reply(&PingRequest {}, None).await?;
            trace!("ping successful");
            let version = self.version().await?;
            info!("coordinator version: {:?}", version);
            return Ok(());
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

    async fn version(&self) -> Result<VersionResponse, CoordinatorError> {
        let version: VersionResponse = self.request_with_reply(&VersionRequest {}, None).await?;
        Ok(version)
    }

    async fn reset(&self, reset_type: ResetType) -> Result<(), CoordinatorError> {
        trace!("reset with reset type {:?}", reset_type);
        self.request(&ResetRequest { reset_type }).await?;
        Ok::<(), CoordinatorError>(())
    }

    async fn set_led(&self, led_status: LedStatus) -> Result<(), CoordinatorError> {
        trace!("setting LED to {:?}", led_status);
        //TODO: const firmwareControlsLed = parseInt(this.version.revision) >= 20211029;
        let firmware_controls_led = true;

        let command = match led_status {
            LedStatus::Disable => {
                if firmware_controls_led {
                    LedControlRequest {
                        led_id: 0xff,
                        mode: 0,
                    }
                } else {
                    LedControlRequest { led_id: 3, mode: 0 }
                }
            }
            LedStatus::On => LedControlRequest { led_id: 3, mode: 1 },
            LedStatus::Off => LedControlRequest { led_id: 3, mode: 0 },
        };

        self.request(&command).await
    }

    async fn change_channel(&self, channel: u8) -> Result<(), CoordinatorError> {
        info!("changing channel to {}", channel);

        let command = ManagementNetworkUpdateRequest {
            destination_address: 0xffff,
            destination_address_mode: AddressMode::AddrBroadcast as u16,
            channel_mask: [channel]
                .into_iter()
                .reduce(|a, c| a + (1 << c))
                .ok_or(CoordinatorError::InvalidChannel)? as u32, //TODO: very likely wrong, check this
            scan_duration: 0xfe,
            scan_count: 0,
            network_manager_address: 0,
        };

        self.request(&command).await
    }

    async fn set_transmit_power(&self, power: i8) -> Result<(), CoordinatorError> {
        info!("setting transmit power to {}", power);
        let command = StackTuneRequest {
            operation: 0,
            value: power,
        };

        self.request(&command).await
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
        let command = ManagementPermitJoinRequest {
            address_mode: address_mode as u16,
            destination_address,
            duration: seconds
                .as_secs()
                .try_into()
                .map_err(|_| CoordinatorError::DurationTooLong)?,
            tc_significance: 0,
        };

        self.request(&command).await
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

        let command = ExitRouteDiscRequest {
            destination_address: address.unwrap_or(0),
            options: 0,
            radius: af::DEFAULT_RADIUS,
        };

        self.request(&command).await
    }

    async fn set_on_event(&mut self, on_zigbee_event: OnEvent) -> Result<(), CoordinatorError> {
        self.on_zigbee_event
            .lock()
            .await
            .replace(Box::new(on_zigbee_event));
        Ok(())
    }

    async fn device_info(&self) -> Result<GetDeviceInfoResponse, CoordinatorError> {
        info!("getting device info...");
        let device_info: GetDeviceInfoResponse = self
            .request_with_reply(&GetDeviceInfoRequest {}, None)
            .await?;
        Ok(device_info)
    }
}
