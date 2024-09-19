use super::{SerialThreadError, SimpleSerial};
use crate::{
    coordinator::CoordinatorError,
    subscription::SubscriptionService,
    utils::{error, trace},
};
use futures::StreamExt;
use futures::{channel::mpsc, executor::block_on, SinkExt};
use futures::{
    channel::mpsc::{Receiver, Sender},
    lock::Mutex,
};
use std::{sync::Arc, thread::JoinHandle};

const DEFAULT_READ_TIMEOUT_MS: u64 = 10;

// Simplest possible serial port implementation
pub struct SimpleSerialPort<P> {
    path: String,
    baud_rate: u32,
    // from the coordinator to the serial port
    to_serial: (Option<Sender<P>>, Option<Receiver<P>>),
    read_thread: Option<JoinHandle<Result<(), SerialThreadError>>>,
    write_thread: Option<JoinHandle<Result<(), SerialThreadError>>>,
    subscription_service: Arc<Mutex<SubscriptionService<P>>>,
}

impl<
        P: for<'a> TryFrom<&'a [u8]>
            + ToOwned<Owned = P>
            + ToSerial
            + PartialEq
            + std::fmt::Debug
            + Clone
            + Send
            + 'static,
    > SimpleSerialPort<P>
{
    pub fn new(
        path: &str,
        baud_rate: u32,
        subscription_service: Arc<Mutex<SubscriptionService<P>>>,
    ) -> Result<Self, CoordinatorError> {
        let to_serial = mpsc::channel(20);
        let to_serial = (Some(to_serial.0), Some(to_serial.1));
        let mut s = SimpleSerialPort {
            path: path.to_string(),
            baud_rate,
            to_serial,
            read_thread: None,
            write_thread: None,
            subscription_service,
        };
        s.start()?;
        Ok(s)
    }

    fn start(&mut self) -> Result<(), CoordinatorError> {
        let mut read = serialport::new(self.path.clone(), self.baud_rate)
            .timeout(std::time::Duration::from_millis(DEFAULT_READ_TIMEOUT_MS))
            .open()
            .map_err(|e| CoordinatorError::SerialOpen(e.to_string()))?;
        let mut write = read
            .try_clone()
            .map_err(|e| CoordinatorError::SerialOpen(e.to_string()))?;

        let subscription_service = self.subscription_service.clone();
        let mut receive_from_serial_send_to_channel = move || -> Result<(), SerialThreadError> {
            loop {
                let mut buffer = [0u8; 256];
                let len = match read.read(&mut buffer) {
                    Ok(r) => Ok(r),
                    Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        continue;
                    }
                    Err(e) => return Err(SerialThreadError::SerialRead(e.to_string())),
                }?;
                if let Some(start_of_frame_position) = buffer.iter().position(|&x| x == 0xfe) {
                    let packet: P = P::try_from(&buffer[start_of_frame_position..len])
                        .map_err(|_| SerialThreadError::PacketParse)?
                        .to_owned();
                    trace!("<<< {:?}", packet);
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
        let mut receive_from_channel_send_to_serial = move || -> Result<(), SerialThreadError> {
            block_on(async {
                while let Some(packet) = rx.next().await {
                    trace!(">>> {:?}", packet);
                    packet
                        .to_serial(&mut *write)
                        .map_err(|e| SerialThreadError::SerialWrite(format!("{:?}", e)))?;
                }
                Ok::<(), SerialThreadError>(())
            })?;
            Ok(())
        };
        self.read_thread.replace(std::thread::spawn(move || {
            receive_from_serial_send_to_channel()
                .inspect_err(|e| error!("receive_from_serial_send_to_channel: {:?}", e))
        }));
        self.write_thread.replace(std::thread::spawn(move || {
            receive_from_channel_send_to_serial()
                .inspect_err(|e| error!("receive_from_channel_send_to_serial: {:?}", e))
        }));
        Ok(())
    }
}

impl<P: Clone> SimpleSerial<P> for SimpleSerialPort<P> {
    type Sender = Sender<P>;
    type Receiver = Receiver<P>;

    async fn write(&mut self, packet: &P) -> Result<(), SerialThreadError> {
        let tx = self
            .to_serial
            .0
            .as_mut()
            .ok_or(SerialThreadError::SerialChannelMissing)?;
        tx.send(packet.clone())
            .await
            .map_err(|_e| SerialThreadError::SerialChannel)
    }
}

pub trait ToSerial {
    fn to_serial<W: std::io::Write + ?Sized>(&self, writer: &mut W) -> Result<(), std::io::Error>;
}
