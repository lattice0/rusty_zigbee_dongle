use super::{SerialThreadError, SimpleSerial};
use crate::{
    coordinator::CoordinatorError,
    unpi::{LenTypeInfo, MessageType, Subsystem, UnpiPacket},
    utils::log,
};
use futures::channel::mpsc::{Receiver, Sender};
use futures::StreamExt;
use futures::{channel::mpsc, executor::block_on, SinkExt};
use std::thread::JoinHandle;

type Container = Vec<u8>;

const DEFAULT_READ_TIMEOUT_MS: u64 = 10;
const DEFAULT_WRITE_TIMEOUT_MS: u64 = 10;

// Simplest possible serial port implementation
pub struct SimpleSerialPort {
    path: String,
    baud_rate: u32,
    // from the serial port to the coordinator
    serial_to: (
        Option<Sender<UnpiPacket<Container>>>,
        Option<Receiver<UnpiPacket<Container>>>,
    ),
    // from the coordinator to the serial port
    to_serial: (
        Option<Sender<UnpiPacket<Container>>>,
        Option<Receiver<UnpiPacket<Container>>>,
    ),
    read_thread: Option<JoinHandle<Result<(), SerialThreadError>>>,
    write_thread: Option<JoinHandle<Result<(), SerialThreadError>>>,
}

impl SimpleSerialPort {
    pub fn new(path: &str, baud_rate: u32) -> Result<Self, CoordinatorError> {
        let serial_to = mpsc::channel(10);
        let serial_to = (Some(serial_to.0), Some(serial_to.1));
        let to_serial = mpsc::channel(10);
        let to_serial = (Some(to_serial.0), Some(to_serial.1));
        Ok(SimpleSerialPort {
            path: path.to_string(),
            baud_rate,
            serial_to,
            to_serial,
            read_thread: None,
            write_thread: None,
        })
    }
}

impl SimpleSerial for SimpleSerialPort {
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
        let mut tx = self
            .serial_to
            .0
            .take()
            .ok_or(CoordinatorError::SerialChannelMissing)?;
        let receive_from_serial_send_to_channel = move || -> Result<(), SerialThreadError> {
            loop {
                let mut buffer = [0u8; 256];
                let len = read
                    .read(&mut buffer)
                    .map_err(|_e| SerialThreadError::SerialRead)?;
                let packet: UnpiPacket<Vec<u8>> = UnpiPacket::from_payload_owned(
                    (&buffer[..len].to_vec(), LenTypeInfo::OneByte),
                    (MessageType::SREQ, Subsystem::Sys),
                    0,
                )
                .map_err(|_| SerialThreadError::MalformedPacket)?;
                log!("<<< {:?}", packet);
                block_on(tx.send(packet)).map_err(|_e| SerialThreadError::SerialWrite)?;
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
                        .map_err(|_e| SerialThreadError::SerialWrite)
                        .unwrap();
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

    async fn read(&mut self) -> Result<UnpiPacket<Vec<u8>>, CoordinatorError> {
        let rx = self
            .serial_to
            .1
            .as_mut()
            .ok_or(CoordinatorError::SerialChannelMissing)?;
        rx.next()
            .await
            .ok_or(CoordinatorError::NoResponse)
            .map_err(|_e| CoordinatorError::SerialRead)
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
