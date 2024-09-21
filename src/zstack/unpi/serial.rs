use super::{commands::CommandRequest, MessageType, SUnpiPacket, Subsystem, UnpiPacket};
use crate::zstack::unpi::commands::CommandResponse;
use crate::{
    coordinator::CoordinatorError,
    serial::{simple_serial_port::ToSerial, SimpleSerial},
    subscription::{Action, Predicate, Subscription, SubscriptionService},
    utils::map::MapError,
    zstack::unpi::MAX_PAYLOAD_SIZE,
};
use crate::{serial::SerialThreadError, utils::info};
use deku::{DekuReader, DekuWriter};
use futures::{
    channel::oneshot::{self, Receiver, Sender},
    lock::Mutex,
};
use serialport::SerialPort;
use std::io::Cursor;
use std::sync::Arc;

// reusable request function
pub async fn request<R: CommandRequest + DekuWriter, S: SimpleSerial<SUnpiPacket>>(
    packet: &SUnpiPacket,
    serial: Arc<Mutex<S>>,
) -> Result<(), UnpiCommandError> {
    serial.lock().await.write(packet).await?;
    Ok(())
}

pub async fn request_with_reply<
    R: CommandRequest + DekuWriter,
    S: SimpleSerial<SUnpiPacket>,
    Res: CommandResponse + for<'de> DekuReader<'de>,
>(
    packet: &SUnpiPacket,
    serial: Arc<Mutex<S>>,
    subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
    timeout: Option<std::time::Duration>,
) -> Result<Res, UnpiCommandError> {
    let wait = wait_for(
        R::Response::id(),
        MessageType::SRESP,
        R::Response::subsystem(),
        subscriptions.clone(),
        timeout,
    );
    let send = async {
        let mut s = serial.lock().await;
        s.write(packet)
            .await
            .map_err(UnpiCommandError::Serial)
    };
    futures::try_join!(send, wait).map(|(_, packet)| packet.to_command_response())?
}

// reusable wait_for function
pub async fn wait_for(
    command_id: u8,
    message_type: MessageType,
    subsystem: Subsystem,
    subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
    _timeout: Option<std::time::Duration>,
) -> Result<SUnpiPacket, UnpiCommandError> {
    // let command = get_command_by_name(&subsystem, name)
    //     .ok_or(UnpiCommandError::NoCommandWithName(name.to_string()))?;
    let (tx, rx): (Sender<SUnpiPacket>, Receiver<SUnpiPacket>) = oneshot::channel();
    {
        let mut s = subscriptions.lock().await;
        let subscription = Subscription::SingleShot(
            Predicate(Box::new(move |packet: &SUnpiPacket| {
                packet.type_subsystem == (message_type, subsystem) && packet.command == command_id
            })),
            Action(Box::new(move |packet: &SUnpiPacket| {
                let _ = tx.send(packet.clone());
            })),
        );
        s.subscribe(subscription);
    }

    rx.await.map_err(|_| UnpiCommandError::SubscriptionError)
}

impl<T> UnpiPacket<T>
where
    T: AsRef<[u8]>,
{
    /// Serialized the packet to the serial port
    pub fn to_serial<S: SerialPort + ?Sized>(
        &self,
        serial: &mut S,
    ) -> Result<(), CoordinatorError> {
        let mut unpi_packet_buffer = [0u8; 256];
        let written = self.to_bytes(&mut unpi_packet_buffer)?;
        serial
            .write_all(&unpi_packet_buffer[0..written])
            .map_err(|_e| CoordinatorError::SerialWrite)?;
        Ok(())
    }

    /// Instantiates a packet from a command and writes it to the serial port
    /// This way we don't have lifetime issues returning the packet referencing the local payload
    #[allow(clippy::needless_borrows_for_generic_args)]
    pub fn from_command_to_serial<R: CommandRequest + DekuWriter, S: SerialPort + ?Sized>(
        command: &R,
        serial: &mut S,
    ) -> Result<(), CoordinatorError> {
        let mut payload_buffer = [0u8; MAX_PAYLOAD_SIZE];
        let original_payload_len = payload_buffer.len();
        //bincode::serialize_into(&mut payload_buffer_writer, command).unwrap();
        let mut cursor = Cursor::new(&mut payload_buffer);
        deku::DekuWriter::to_writer(command, &mut cursor, ()).unwrap();
        let written = original_payload_len - cursor.position() as usize;
        let payload: &[u8] = &payload_buffer[0..written];
        // let h =
        //     UnpiPacket::from_payload((payload, LenTypeInfo::OneByte), type_subsystem, command_id)?;
        payload.to_serial(&mut *serial)?;
        info!(">>> {:?}", command);
        Ok(())
    }

    /// Instantiates a packet from a command and writes it to the serial port
    /// This way we don't have lifetime issues returning the packet referencing the local payload
    // TODO: in the future maybe use a proper async serial port library?
    pub async fn from_command_to_serial_async<
        R: CommandRequest + DekuWriter,
        S: SerialPort + ?Sized,
    >(
        command: &R,
        serial: &mut S,
    ) -> Result<(), CoordinatorError> {
        Self::from_command_to_serial(command, serial)
    }
}

#[derive(Debug)]
pub enum UnpiCommandError {
    NoCommandWithName(String),
    InvalidMessageType,
    SubscriptionError,
    Serial(SerialThreadError),
    Io(std::io::Error),
    Map(MapError),
    InvalidResponse,
    Bincode,
}

impl From<std::io::Error> for UnpiCommandError {
    fn from(e: std::io::Error) -> Self {
        UnpiCommandError::Io(e)
    }
}

impl From<SerialThreadError> for UnpiCommandError {
    fn from(e: SerialThreadError) -> Self {
        UnpiCommandError::Serial(e)
    }
}

impl From<MapError> for UnpiCommandError {
    fn from(e: MapError) -> Self {
        UnpiCommandError::Map(e)
    }
}
