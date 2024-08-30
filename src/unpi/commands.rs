use crate::coordinator::{Coordinator, CoordinatorError};
use std::io::Write;
use super::{MessageType, Subsystem};

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: &'static str,
    pub id: u8,
    pub command_type: MessageType,
    pub request: &'static [(&'static str, ParameterType)],
    pub response: &'static [(&'static str, ParameterType)],
}

impl Command {
    pub fn to_payload_bytes(
        &self,
        values: &[(&'static str, ParameterValue)],
        output: &mut [u8],
    ) -> Result<(), CoordinatorError> {
        // values.iter().reduce(|acc, x| {

        // })
        values
            .iter()
            .for_each(|(_name, value)| value.to_bytes(output));
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub enum ParameterType {
    U8,
    U16,
}

#[derive(Debug, PartialEq)]
pub enum ParameterValue {
    U8(u8),
    U16(u16),
}

impl ParameterValue {
    pub fn to_bytes(&self, output: &mut [u8]) {
        match self {
            ParameterValue::U8(v) => output[0..1].copy_from_slice(&[*v]),
            ParameterValue::U16(v) => output[0..2].copy_from_slice(&v.to_le_bytes()),
        }
    }
}

pub const SUBSYSTEMS: &[(Subsystem, &[Command])] = &[(Subsystem::Util, COMMANDS_UTIL)];

pub fn get_command_by_name(subsystem: &Subsystem, name: &str) -> Option<&'static Command> {
    SUBSYSTEMS
        .iter()
        .find(|(s, _)| s == subsystem)
        .and_then(|(_, cmds)| cmds.iter().find(|c| c.name == name))
}

pub fn get_command_by_id(subsystem: &Subsystem, id: u8) -> Option<&'static Command> {
    SUBSYSTEMS
        .iter()
        .find(|(s, _)| s == subsystem)
        .and_then(|(_, cmds)| cmds.iter().find(|c| c.id == id))
}

pub const COMMANDS_UTIL: &[Command] = &[Command {
    name: "led_control",
    id: 10,
    command_type: MessageType::SREQ,
    request: &[("led_id", ParameterType::U8), ("mode", ParameterType::U8)],
    response: &[("status", ParameterType::U8)],
}];

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_command_by_name() {
        assert_eq!(
            get_command_by_name(&Subsystem::Util, "led_control"),
            Some(&Command {
                name: "led_control",
                id: 10,
                command_type: MessageType::SREQ,
                request: todo!(),
                response: todo!(),
            })
        );
    }

    #[test]
    fn test_get_command_by_id() {
        assert_eq!(
            get_command_by_id(&Subsystem::Util, 10),
            Some(&Command {
                name: "led_control",
                id: 10,
                command_type: MessageType::SREQ,
                request: todo!(),
                response: todo!(),
            })
        );
    }
}
