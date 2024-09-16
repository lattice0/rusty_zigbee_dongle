use super::{
    commands::{Command, ParametersValueMap},
    MessageType, SUnpiPacket, Subsystem, UnpiPacket,
};
use crate::{
    coordinator::CoordinatorError,
    parameters::ParameterValue,
    serial::SimpleSerial,
    subscription::{Action, Predicate, Subscription, SubscriptionService},
    utils::map::MapError,
    zstack::unpi::{LenTypeInfo, MAX_PAYLOAD_SIZE},
};
use crate::{parameters::ParameterError, serial::SerialThreadError, utils::info};
use futures::{
    channel::oneshot::{self, Receiver, Sender},
    lock::Mutex,
};
use serialport::SerialPort;
use std::sync::Arc;

// reusable request function
pub async fn request<S: SimpleSerial<SUnpiPacket>>(
    name: &str,
    subsystem: Subsystem,
    parameters: &[(&'static str, ParameterValue)],
    serial: Arc<Mutex<S>>,
) -> Result<(), UnpiCommandError> {
    let command = get_command_by_name(&subsystem, name)
        .ok_or(UnpiCommandError::NoCommandWithName(name.to_string()))?;
    let packet = SUnpiPacket::from_command_owned(
        LenTypeInfo::OneByte,
        (command.command_type, subsystem),
        parameters,
        command,
    )?;
    serial.lock().await.write(&packet).await?;
    Ok(())
}

pub async fn request_with_reply<S: SimpleSerial<SUnpiPacket>>(
    name: &str,
    subsystem: Subsystem,
    parameters: &[(&'static str, ParameterValue)],
    serial: Arc<Mutex<S>>,
    subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
    timeout: Option<std::time::Duration>,
) -> Result<ParametersValueMap, UnpiCommandError> {
    let command = get_command_by_name(&subsystem, name)
        .ok_or(UnpiCommandError::NoCommandWithName(name.to_string()))?;
    let packet = SUnpiPacket::from_command_owned(
        LenTypeInfo::OneByte,
        (command.command_type, subsystem),
        parameters,
        command,
    )?;
    let wait = wait_for(
        name,
        MessageType::SRESP,
        subsystem,
        subscriptions.clone(),
        timeout,
    );
    let send = async {
        let mut s = serial.lock().await;
        s.write(&packet)
            .await
            .map_err(|e| UnpiCommandError::Serial(e))
    };
    futures::try_join!(send, wait)
        .map(|(_, (packet, command))| command.read_and_fill(packet.payload.as_slice()))?
}

// reusable wait_for function
pub async fn wait_for(
    name: &str,
    message_type: MessageType,
    subsystem: Subsystem,
    subscriptions: Arc<Mutex<SubscriptionService<SUnpiPacket>>>,
    _timeout: Option<std::time::Duration>,
) -> Result<(SUnpiPacket, Command), UnpiCommandError> {
    let command = get_command_by_name(&subsystem, name)
        .ok_or(UnpiCommandError::NoCommandWithName(name.to_string()))?;
    let (tx, rx): (Sender<SUnpiPacket>, Receiver<SUnpiPacket>) = oneshot::channel();
    {
        let mut s = subscriptions.lock().await;
        let subscription = Subscription::SingleShot(
            Predicate(Box::new(move |packet: &SUnpiPacket| {
                packet.type_subsystem == (message_type, subsystem) && packet.command == command.id
            })),
            Action(Box::new(move |packet: &SUnpiPacket| {
                let _ = tx.send(packet.clone());
            })),
        );
        s.subscribe(subscription);
    }
    let mut response_command = command.clone();
    // We rewrite the command as being a response if it was a request
    match response_command.command_type {
        MessageType::AREQ => response_command.command_type = MessageType::AREQ,
        MessageType::SREQ => response_command.command_type = MessageType::SRESP,
        _ => return Err(UnpiCommandError::InvalidMessageType),
    }
    Ok((
        rx.await.map_err(|_| UnpiCommandError::SubscriptionError)?,
        response_command,
    ))
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
    pub fn from_command_to_serial<S: SerialPort + ?Sized>(
        command_id: u8,
        command: &Command,
        parameters: &[(&'static str, ParameterValue)],
        type_subsystem: (MessageType, Subsystem),
        serial: &mut S,
    ) -> Result<(), CoordinatorError> {
        let mut payload_buffer = [0u8; MAX_PAYLOAD_SIZE];
        let written = command.fill_and_write(parameters, &mut payload_buffer)?;
        let payload: &[u8] = &payload_buffer[0..written];
        let h =
            UnpiPacket::from_payload((payload, LenTypeInfo::OneByte), type_subsystem, command_id)?;
        h.to_serial(&mut *serial)?;
        info!(">>> {:?}", h);
        Ok(())
    }

    /// Instantiates a packet from a command and writes it to the serial port
    /// This way we don't have lifetime issues returning the packet referencing the local payload
    // TODO: in the future maybe use a proper async serial port library?
    pub async fn from_command_to_serial_async<S: SerialPort + ?Sized>(
        command_id: u8,
        command: &Command,
        parameters: &[(&'static str, ParameterValue)],
        type_subsystem: (MessageType, Subsystem),
        serial: &mut S,
    ) -> Result<(), CoordinatorError> {
        Self::from_command_to_serial(command_id, command, parameters, type_subsystem, serial)
    }
}

#[derive(Debug)]
pub enum UnpiCommandError {
    NoCommandWithName(String),
    InvalidMessageType,
    SubscriptionError,
    Parameter(ParameterError),
    Serial(SerialThreadError),
    Io(std::io::Error),
    Map(MapError),
    InvalidResponse,
}

impl From<ParameterError> for UnpiCommandError {
    fn from(e: ParameterError) -> Self {
        UnpiCommandError::Parameter(e)
    }
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
