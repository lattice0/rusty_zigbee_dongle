use crate::{
    coordinator::CoordinatorError,
    unpi::{
        commands::{Command, ParameterValue},
        LenTypeInfo, MessageType, Subsystem, UnpiPacket, UnpiPacketSink, MAX_PAYLOAD_SIZE,
    },
    utils::log,
};
use futures::channel::oneshot;
use futures::{
    channel::oneshot::{Receiver, Sender},
    lock::Mutex,
    AsyncReadExt, AsyncWrite, AsyncWriteExt,
};
use serialport::SerialPort;
use std::{future::Future, sync::Arc};
use futures::task::Spawn;

pub struct SimpleSerialPort {
    read: Arc<Mutex<Box<dyn SerialPort>>>,
    write: Arc<Mutex<Box<dyn SerialPort>>>,
    tx: Sender<UnpiPacket<'static>>,
    rx: Receiver<UnpiPacket<'static>>,
}

impl SimpleSerialPort {
    pub fn new(path: &str, baud_rate: u32) -> Result<Self, CoordinatorError> {
        let (tx, rx) = oneshot::channel();
        Ok(SimpleSerialPort {
            read: Arc::new(Mutex::new(
                serialport::new(path, baud_rate)
                    .timeout(std::time::Duration::from_millis(10))
                    .open()
                    .map_err(|_e| CoordinatorError::SerialOpen)?,
            )),
            write: Arc::new(Mutex::new(
                serialport::new(path, baud_rate)
                    .timeout(std::time::Duration::from_millis(10))
                    .open()
                    .map_err(|_e| CoordinatorError::SerialOpen)?,
            )),
            tx,
            rx,
        })
    }

    pub async fn start(&self) -> Result<(), CoordinatorError> {
        let mut read = self.read.lock().await;
        let mut write = self.write.lock().await;
        let (tx, rx) = (self.tx, self.rx);
        let mut buffer = [0u8; 256];
        let receive = async {
            loop {
                let len = read.read(&mut buffer).map_err(|_e| CoordinatorError::SerialRead)?;
                let packet = UnpiPacket::from_payload(
                    (&buffer[..len], LenTypeInfo::OneByte),
                    (MessageType::SREQ, Subsystem::Sys),
                    0,
                )?;
                log!("<<< {:?}", packet);
                tx.send(packet).map_err(|_e| CoordinatorError::SerialWrite)?;
            }
        };
        let send = async {
            loop {
                let packet = rx.await.map_err(|_e| CoordinatorError::SerialWrite)?;
                let mut buffer = [0u8; 256];
                packet.to_bytes(&mut buffer)?;
                write.write_all(buffer).map_err(|_e| CoordinatorError::SerialWrite)?;
                
            }
        };
        todo!()
    }
}

impl<'a> UnpiPacket<'a> {
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
        log!(">>> {:?}", h);
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
