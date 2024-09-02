use super::{subsystems::SUBSYSTEMS, MessageType, Subsystem};
use crate::{
    coordinator::CoordinatorError,
    utils::{log, Map},
};
use std::io::Write;

#[derive(Debug, PartialEq)]
/// Represents a command in the UNPI protocol.
pub struct Command {
    pub name: &'static str,
    pub id: u8,
    pub command_type: MessageType,
    pub request: Map<&'static str, ParameterType>,
    pub response: Map<&'static str, ParameterType>,
}

impl Command {
    /// Fills the buffer with the parameters, failing if one of them isn't supposed to be there
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
    I8,
}

#[derive(Debug, PartialEq)]
pub enum ParameterValue {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
}

impl PartialEq<ParameterType> for ParameterValue {
    fn eq(&self, other: &ParameterType) -> bool {
        match self {
            ParameterValue::U8(_) => other == &ParameterType::U8,
            ParameterValue::U16(_) => other == &ParameterType::U16,
            ParameterValue::U32(_) => other == &ParameterType::U32,
            ParameterValue::I8(_) => other == &ParameterType::I8,
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
            //TODO: i8 to u8?
            ParameterValue::I8(v) => output.write_all(&[*v as u8])?,
        }
        Ok(len - output.len())
    }
}

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
