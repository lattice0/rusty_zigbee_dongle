use super::{
    parameters::{ParameterType, ParameterValue},
    subsystems::SUBSYSTEMS,
    MessageType, Subsystem,
};
use crate::{
    coordinator::CoordinatorError,
    utils::{map::StaticMap, slice_reader::SliceReader},
};

pub const MAX_COMMAND_SIZE: usize = 15;
pub type ParametersValueMap = StaticMap<MAX_COMMAND_SIZE, &'static str, ParameterValue>;
pub type ParametersTypeMap = StaticMap<MAX_COMMAND_SIZE, &'static str, ParameterType>;

#[derive(Debug, PartialEq)]
/// Represents a command in the UNPI protocol.
pub struct Command {
    pub name: &'static str,
    pub id: u8,
    pub command_type: MessageType,
    pub request: Option<ParametersTypeMap>,
    pub response: Option<ParametersTypeMap>,
}

impl Command {
    /// Fills the buffer with the parameters, failing if one of them isn't supposed to be there
    pub fn fill_and_write(
        &self,
        parameters: &[(&'static str, ParameterValue)],
        mut output: &mut [u8],
    ) -> Result<usize, CoordinatorError> {
        let len = output.len();
        if let Some(request) = self.request.as_ref() {
            // Let's fill the values and match against the template in self.request, just for safety
            parameters.iter().try_for_each(|(name, value)| {
                // Find parameter in request
                let parameter_type = request
                    .get(name)
                    .ok_or(CoordinatorError::NoCommandWithName(name.to_string()))?;
                if request.contains_key(name) {
                    // Only writes if we match the parameter type
                    let written = value.match_and_write(parameter_type, output)?;
                    let new_output = std::mem::take(&mut output);
                    output = &mut new_output[written..];
                } else {
                    return Err(CoordinatorError::NoCommandWithName(name.to_string()));
                }

                //log!("COMMAND: {}, VALUE {:?}", name, value);
                Ok::<(), CoordinatorError>(())
            })?;
        }
        Ok(len - output.len())
    }

    pub fn read_and_fill(&self, input: &[u8]) -> Result<ParametersValueMap, CoordinatorError> {
        let mut reader = SliceReader(input);
        let response = self
            .response
            .as_ref()
            .ok_or(CoordinatorError::ResponseMismatch)?;
        let parameters = match self.name {
            // Special case for get_device_info, where num_assoc_devices specifies the list length before it comes
            "get_device_info" => {
                let mut parameters: ParametersValueMap = Default::default();
                let status = reader.read_u8()?;
                let ieee_addr = reader.read_u8_array(8)?;
                let short_addr = reader.read_u16_le()?;
                let device_type = reader.read_u8()?;
                let device_state = reader.read_u8()?;
                let num_assoc_devices = reader.read_u8()?;
                let assoc_devices_list = reader.read_u16_array(16)?;
                parameters.insert("status", ParameterValue::U8(status))?;
                parameters.insert("ieee_addr", ParameterValue::IeeAddress(ieee_addr))?;
                parameters.insert("short_addr", ParameterValue::U16(short_addr))?;
                parameters.insert("device_type", ParameterValue::U8(device_type))?;
                parameters.insert("device_state", ParameterValue::U8(device_state))?;
                parameters.insert("num_assoc_devices", ParameterValue::U8(num_assoc_devices))?;
                parameters.insert(
                    "assoc_devices_list",
                    ParameterValue::ListU16(assoc_devices_list),
                )?;
                parameters
            }
            _ => {
                let mut parameters: ParametersValueMap = Default::default();
                response.iter().try_for_each(|(name, parameter_type)| {
                    let value = parameter_type.from_slice_reader(&mut reader)?;
                    parameters.insert(name, value)?;
                    Ok::<(), CoordinatorError>(())
                })?;
                parameters
            }
        };
        Ok(parameters)
    }
}

/// Get a command by name, linear (kinda slow) search over the static slice
pub fn get_command_by_name(subsystem: &Subsystem, name: &str) -> Option<&'static Command> {
    SUBSYSTEMS
        .iter()
        .find(|(s, _)| s == subsystem)
        .and_then(|(_, cmds)| cmds.iter().find(|c| c.name == name))
}

/// Get a command by name, linear (kinda slow) search over the static slice
pub fn get_command_by_id(subsystem: &Subsystem, id: u8) -> Option<&'static Command> {
    SUBSYSTEMS
        .iter()
        .find(|(s, _)| s == subsystem)
        .and_then(|(_, cmds)| cmds.iter().find(|c| c.id == id))
}

#[derive(Debug)]
pub enum ParameterError {
    InvalidParameter,
    Io(String),
    NoCommandWithName,
    Unreachable,
    MissingListLength,
}

impl From<std::io::Error> for ParameterError {
    fn from(e: std::io::Error) -> Self {
        ParameterError::Io(e.to_string())
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
