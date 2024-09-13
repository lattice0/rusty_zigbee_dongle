use super::{
    commands::{get_command_by_name, Command},
    MessageType, SUnpiPacket, Subsystem, UnpiPacket,
};
use crate::{
    coordinator::CoordinatorError, parameters::ParameterValue, subscription::{Action, Predicate, Subscription, SubscriptionService}, zstack::unpi::{LenTypeInfo, MAX_PAYLOAD_SIZE}
};
use futures::{
    channel::oneshot::{self, Receiver, Sender},
    lock::Mutex,
};
use serialport::SerialPort;
use std::sync::Arc;
use crate::utils::info;

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
}
