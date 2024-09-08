use super::{SerialThreadError, SubscriptionSerial};
use crate::{
    coordinator::CoordinatorError,
    subscription::SubscriptionService,
    unpi::{LenTypeInfo, UnpiPacket},
    utils::log,
};
use futures::StreamExt;
use futures::{channel::mpsc, executor::block_on, SinkExt};
use futures::{
    channel::mpsc::{Receiver, Sender},
    lock::Mutex,
};
use std::{sync::Arc, thread::JoinHandle};

type Container = Vec<u8>;

const DEFAULT_READ_TIMEOUT_MS: u64 = 10;

// Simplest possible serial port implementation
pub struct SimpleSerialPort {
    path: String,
    baud_rate: u32,
    // from the coordinator to the serial port
    to_serial: (
        Option<Sender<UnpiPacket<Container>>>,
        Option<Receiver<UnpiPacket<Container>>>,
    ),
    read_thread: Option<JoinHandle<Result<(), SerialThreadError>>>,
    write_thread: Option<JoinHandle<Result<(), SerialThreadError>>>,
    subscription_service: Arc<Mutex<SubscriptionService<UnpiPacket<Container>>>>,
}

impl SimpleSerialPort {
    pub fn new(
        path: &str,
        baud_rate: u32,
        subscription_service: Arc<Mutex<SubscriptionService<UnpiPacket<Container>>>>,
    ) -> Result<Self, CoordinatorError> {
        let to_serial = mpsc::channel(10);
        let to_serial = (Some(to_serial.0), Some(to_serial.1));
        Ok(SimpleSerialPort {
            path: path.to_string(),
            baud_rate,
            to_serial,
            read_thread: None,
            write_thread: None,
            subscription_service,
        })
    }
}

impl SubscriptionSerial for SimpleSerialPort {
    type Sender = Sender<UnpiPacket<Container>>;
    type Receiver = Receiver<UnpiPacket<Container>>;

    fn start(&mut self) -> Result<(), CoordinatorError> {
        let mut read = serialport::new(self.path.clone(), self.baud_rate)
            .timeout(std::time::Duration::from_millis(DEFAULT_READ_TIMEOUT_MS))
            .open()
            .map_err(|e| CoordinatorError::SerialOpen(e.to_string()))?;
        let mut write = read
            .try_clone()
            .map_err(|e| CoordinatorError::SerialOpen(e.to_string()))?;

        let subscription_service = self.subscription_service.clone();
        let receive_from_serial_send_to_channel = move || -> Result<(), SerialThreadError> {
            loop {
                let mut buffer = [0u8; 256];
                let len = read
                    .read(&mut buffer)
                    .map_err(|_e| SerialThreadError::SerialRead)?;
                if let Some(start_of_frame_position) = buffer.iter().position(|&x| x == 0xfe) {
                    let packet: UnpiPacket<Vec<u8>> = UnpiPacket::try_from((
                        &buffer[start_of_frame_position..len],
                        LenTypeInfo::OneByte,
                    ))
                    .map_err(|_e| SerialThreadError::PacketParse)?
                    .to_owned();
                    log!("<<< {:?}", packet);
                    let send = async { subscription_service.lock().await.notify(packet) };
                    block_on(send).map_err(|_| SerialThreadError::SubscriptionWrite)?;
                }
            }
        };
        let mut rx = self
            .to_serial
            .1
            .take()
            .ok_or(CoordinatorError::SerialChannelMissing)?;
        let receive_from_channel_send_to_serial = move || -> Result<(), SerialThreadError> {
            block_on(async {
                while let Some(packet) = rx.next().await {
                    log!(">>> {:?}", packet);
                    packet
                        .to_serial(&mut *write)
                        .map_err(|_e| SerialThreadError::SerialWrite)?;
                }
                Ok::<(), SerialThreadError>(())
            })?;
            Ok(())
        };
        self.read_thread
            .replace(std::thread::spawn(receive_from_serial_send_to_channel));
        self.write_thread
            .replace(std::thread::spawn(receive_from_channel_send_to_serial));
        Ok(())
    }

    async fn write(&mut self, packet: &UnpiPacket<Vec<u8>>) -> Result<(), CoordinatorError> {
        let tx = self
            .to_serial
            .0
            .as_mut()
            .ok_or(CoordinatorError::SerialChannelMissing)?;
        tx.send(packet.clone())
            .await
            .map_err(|_e| CoordinatorError::SerialWrite)
    }
}
