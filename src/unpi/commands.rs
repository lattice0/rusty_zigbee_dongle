use super::{MessageType, Subsystem};
use crate::{
    coordinator::CoordinatorError,
    utils::{log, Map},
};
use std::io::Write;

#[derive(Debug, PartialEq)]
pub struct Command {
    pub name: &'static str,
    pub id: u8,
    pub command_type: MessageType,
    pub request: Map<&'static str, ParameterType>,
    pub response: Map<&'static str, ParameterType>,
}

impl Command {
    pub fn fill_and_write(
        &self,
        parameters: &[(&'static str, ParameterValue)],
        mut output: &mut [u8],
    ) -> Result<usize, CoordinatorError> {
        let len = output.len();
        // Let's fill the values and match against the template in self.request, just for safety
        parameters.iter().try_for_each(|(name, value)| {
            // Find parameter in request
            let parameter_type = self
                .request
                .get(name)
                .ok_or(CoordinatorError::NoCommandWithName)?;
            if self.request.contains_key(name) {
                // Only writes if we match the parameter type
                let written = value.match_and_write(parameter_type, output)?;
                let new_output = std::mem::take(&mut output);
                output = &mut new_output[written..];
            } else {
                return Err(CoordinatorError::NoCommandWithName);
            }

            log!("COMMAND: {}, VALUE {:?}", name, value);
            Ok::<(), CoordinatorError>(())
        })?;
        Ok(len - output.len())
    }
}

#[derive(Debug, PartialEq)]
pub enum ParameterType {
    U8,
    U16,
    U32,
}

#[derive(Debug, PartialEq)]
pub enum ParameterValue {
    U8(u8),
    U16(u16),
    U32(u32),
}

impl PartialEq<ParameterType> for ParameterValue {
    fn eq(&self, other: &ParameterType) -> bool {
        match self {
            ParameterValue::U8(_) => other == &ParameterType::U8,
            ParameterValue::U16(_) => other == &ParameterType::U16,
            ParameterValue::U32(_) => other == &ParameterType::U32,
        }
    }
}

impl ParameterValue {
    pub fn match_and_write(
        &self,
        parameter_type: &ParameterType,
        mut output: &mut [u8],
    ) -> Result<usize, ParameterError> {
        let len = output.len();
        if self != parameter_type {
            return Err(ParameterError::InvalidParameter);
        }
        match self {
            ParameterValue::U8(v) => output.write_all(&[*v])?,
            ParameterValue::U16(v) => output.write_all(&v.to_le_bytes())?,
            ParameterValue::U32(v) => output.write_all(&v.to_le_bytes())?,
        }
        Ok(len - output.len())
    }
}

pub const SUBSYSTEMS: &[(Subsystem, &[Command])] = &[
    (Subsystem::Util, COMMANDS_UTIL),
    (Subsystem::Zdo, COMMANDS_ZDO),
];

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
    request: Map::new(&[("led_id", ParameterType::U8), ("mode", ParameterType::U8)]),
    response: Map::new(&[("status", ParameterType::U8)]),
}];

pub const COMMANDS_ZDO: &[Command] = &[Command {
    name: "mgmtNwkUpdateReq",
    id: 55, // TODO: 0x0038 => 56?? (from zStackAdapter.ts)
    command_type: MessageType::SREQ,
    request: Map::new(&[
        ("dst_addr", ParameterType::U16),
        ("dst_addr_mode", ParameterType::U16),
        ("channel_mask", ParameterType::U32),
        ("scan_duration", ParameterType::U8),
        ("scan_count", ParameterType::U8),
        ("nwk_manager_addr", ParameterType::U16),
    ]),
    response: Map::new(&[("status", ParameterType::U8)]),
}];


#[derive(Debug)]
pub enum ParameterError {
    InvalidParameter,
    Io,
    NoCommandWithName,
}

impl From<std::io::Error> for ParameterError {
    fn from(_: std::io::Error) -> Self {
        ParameterError::Io
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_command_by_name() {
        assert!(matches!(
            get_command_by_name(&Subsystem::Util, "led_control"),
            Some(Command {
                name: "led_control",
                ..
            })
        ));
        assert!(matches!(
            get_command_by_name(&Subsystem::Util, "not_found"),
            None
        ));
    }

    #[test]
    fn test_get_command_by_id() {
        assert!(matches!(
            get_command_by_id(&Subsystem::Util, 10),
            Some(Command {
                name: "led_control",
                id: 10,
                ..
            })
        ));
        assert!(matches!(get_command_by_id(&Subsystem::Util, 11), None));
    }

    #[test]
    fn test_command() {
        let command = get_command_by_name(&Subsystem::Util, "led_control").unwrap();
        let mut buffer = [0; 255];
        let len = command
            .fill_and_write(
                &[
                    ("led_id", ParameterValue::U8(1)),
                    ("mode", ParameterValue::U8(1)),
                ],
                &mut buffer,
            )
            .unwrap();
        assert_eq!(len, 2);
        assert_eq!(&buffer[0..len], [1, 1]);
    }
}
