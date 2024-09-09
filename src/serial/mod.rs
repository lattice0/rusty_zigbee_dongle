use crate::{
    coordinator::CoordinatorError,
    unpi::{
        commands::{Command, ParameterValue},
        LenTypeInfo, MessageType, Subsystem, UnpiPacket, MAX_PAYLOAD_SIZE,
    },
    utils::info,
};
use serialport::SerialPort;
use std::future::Future;

pub mod simple_serial_port;

pub trait SubscriptionSerial {
    type Sender;
    type Receiver;

    fn write(
        &mut self,
        packet: &UnpiPacket<Vec<u8>>,
    ) -> impl Future<Output = Result<(), CoordinatorError>>;

    fn start(&mut self) -> Result<(), CoordinatorError>;
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

#[derive(Debug, PartialEq)]
pub enum SerialThreadError {
    SerialOpen,
    SerialRead,
    SerialWrite,
    MalformedPacket,
    SubscriptionWrite,
    PacketParse,
}
