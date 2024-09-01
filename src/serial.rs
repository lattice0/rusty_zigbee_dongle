use crate::{
    coordinator::CoordinatorError,
    unpi::{
        commands::{Command, ParameterValue},
        LenType, LenTypeInfo, MessageType, Subsystem, UnpiPacket, MAX_PAYLOAD_SIZE,
    },
};
use serialport::SerialPort;

impl<'a> UnpiPacket<'a> {
    pub fn to_serial<S: SerialPort + ?Sized>(
        &self,
        serial: &mut S,
    ) -> Result<(), CoordinatorError> {
        let mut unpi_packet_buffer = [0u8; 256];
        let written = self.to_bytes(&mut unpi_packet_buffer)?;
        serial
            .write_all(&unpi_packet_buffer[0..written])
            .map_err(|e| CoordinatorError::SerialWrite(e.to_string()))?;
        Ok(())
    }

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
        let mut h =
            UnpiPacket::from_payload((payload, LenTypeInfo::OneByte), type_subsystem, command_id)?;
        h.to_serial(&mut *serial)?;
        Ok(())
    }
}
