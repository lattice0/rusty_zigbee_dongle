use crate::{
    coordinator::CoordinatorError,
    unpi::{
        commands::{Command, ParameterValue},
        LenTypeInfo, MessageType, Subsystem, UnpiPacket, UnpiPacketSink, MAX_PAYLOAD_SIZE,
    },
    utils::log,
};
use futures::{channel::mpsc, SinkExt};
use futures::task::Spawn;
use futures::{
    channel::mpsc::{Receiver, Sender},
    lock::Mutex,
    AsyncReadExt, AsyncWrite, AsyncWriteExt,
};
use serialport::SerialPort;
use std::{future::Future, sync::Arc, thread::JoinHandle};

pub struct SimpleSerialPort {
    path: String,
    baud_rate: u32,
    // from the serial port to the coordinator
    serial_to: (
        Option<Sender<UnpiPacket<'static>>>,
        Option<Receiver<UnpiPacket<'static>>>,
    ),
    // from the coordinator to the serial port
    to_serial: (
        Option<Sender<UnpiPacket<'static>>>,
        Option<Receiver<UnpiPacket<'static>>>,
    ),
    read_thread: Option<JoinHandle<()>>,
    write_thread: Option<JoinHandle<()>>,
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

    pub async fn start(&mut self) -> Result<(), CoordinatorError> {
        let mut read = serialport::new(self.path.clone(), self.baud_rate)
            .timeout(std::time::Duration::from_millis(10))
            .open()
            .map_err(|_e| CoordinatorError::SerialOpen)?;
        let mut write = serialport::new(self.path.clone(), self.baud_rate)
            .timeout(std::time::Duration::from_millis(10))
            .open()
            .map_err(|_e| CoordinatorError::SerialOpen)?;

        let tx = self.serial_to.0.take().unwrap();
        let receive_from_serial_send_to_channel = move || {
            let tx = tx;
            loop {
                let mut buffer = [0u8; 256];
                let len = read
                    .read(&mut buffer)
                    .map_err(|_e| CoordinatorError::SerialRead)
                    .unwrap();
                let packet = UnpiPacket::from_payload(
                    (&buffer[..len], LenTypeInfo::OneByte),
                    (MessageType::SREQ, Subsystem::Sys),
                    0,
                )
                .unwrap();
                log!("<<< {:?}", packet);
                tx.send(packet)
                    .map_err(|_e| CoordinatorError::SerialWrite)
                    .unwrap();
            }
        };
        let rx = self.to_serial.1.take().unwrap();
        let receive_from_channel_send_to_serial = || loop {
            let packet = rx.recv().unwrap();
            todo!()
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
