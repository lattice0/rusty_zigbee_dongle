use super::{serial::UnpiCommandError, MessageType, Subsystem};
use crate::{
    parameters::{ParameterType, ParameterValue},
    utils::{map::StaticMap, slice_reader::SliceReader},
};

pub const MAX_COMMAND_SIZE: usize = 15;
pub type ParametersValueMap = StaticMap<MAX_COMMAND_SIZE, &'static str, ParameterValue>;
pub type ParametersTypeMap = StaticMap<MAX_COMMAND_SIZE, &'static str, ParameterType>;

#[derive(Debug, PartialEq, Clone)]
/// Represents a command in the UNPI protocol.
pub struct Command {
    pub name: &'static str,
    pub id: u8,
    pub command_type: MessageType,
    pub request: Option<ParametersTypeMap>,
    pub response: Option<ParametersTypeMap>,
}

pub trait CommandRequest {
    type Response;

    fn id() -> u8;
    fn message_type() -> MessageType;
}

pub trait CommandResponse {
    fn message_type() -> MessageType;
}

pub struct CommandBuffer {
    pub buffer: [u8; 255],
    pub len: usize,
}

pub struct CommandListU16 {
    pub list: [u16; 255],
    pub len: usize,
}

pub struct CommandIeeeAddress {
    pub ieee_address: [u8; 8],
}

#[macro_export]
macro_rules! command {
    // Match a struct declaration with fields
    (
     $id: literal,
     $mty: expr,
     struct $name:ident { $( $field:ident : $type:ty ),* },
     struct $rname:ident { $( $rfield:ident : $rtype:ty ),* },
    ) => {
        #[allow(dead_code)]
        pub struct $name {
            $( $field: $type ),*
        }

        impl $crate::zstack::unpi::commands::CommandRequest for $name {
            type Response = $rname;

            fn id() -> u8 {
                $id
            }

            fn message_type() -> $crate::zstack::unpi::MessageType {
                $mty
            }
        }

        #[allow(dead_code)]
        pub struct $rname {
            $( $rfield: $rtype ),*
        }

        // impl $crate::zstack::unpi::commands::CommandResponse for $rname {
        //     fn message_type() -> $crate::zstack::unpi::MessageType {
        //         $mty
        //     }
        // }

    };
}

impl Command {
    /// Fills the buffer with the parameters, failing if one of them isn't supposed to be there
    pub fn fill_and_write(
        &self,
        parameters: &[(&'static str, ParameterValue)],
        mut output: &mut [u8],
    ) -> Result<usize, UnpiCommandError> {
        let len = output.len();
        if let Some(request) = self.request.as_ref() {
            // Let's fill the values and match against the template in self.request, just for safety
            parameters.iter().try_for_each(|(name, value)| {
                // Find parameter in request
                let parameter_type = request
                    .get(name)
                    .ok_or(UnpiCommandError::NoCommandWithName(name.to_string()))?;
                if request.contains_key(name) {
                    // Only writes if we match the parameter type
                    let written = value.match_and_write(parameter_type, output)?;
                    let new_output = std::mem::take(&mut output);
                    output = &mut new_output[written..];
                } else {
                    return Err(UnpiCommandError::NoCommandWithName(name.to_string()));
                }

                Ok::<(), UnpiCommandError>(())
            })?;
        }
        Ok(len - output.len())
    }

    pub fn read_and_fill(&self, input: &[u8]) -> Result<ParametersValueMap, UnpiCommandError> {
        let mut reader = SliceReader(input);
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
                let assoc_devices_list = reader.read_u16_array(num_assoc_devices as usize)?;
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
                //Asynchronous request
                if let (Some(request), MessageType::AREQ) = (self.request, self.command_type) {
                    let mut parameters: ParametersValueMap = Default::default();
                    request.iter().try_for_each(|(name, parameter_type)| {
                        let value = parameter_type.from_slice_reader(&mut reader)?;
                        parameters.insert(name, value)?;
                        Ok::<(), UnpiCommandError>(())
                    })?;
                    parameters
                } else if let (Some(response), MessageType::SRESP) =
                    (self.response, self.command_type)
                {
                    let mut parameters: ParametersValueMap = Default::default();
                    response.iter().try_for_each(|(name, parameter_type)| {
                        let value = parameter_type.from_slice_reader(&mut reader)?;
                        parameters.insert(name, value)?;
                        Ok::<(), UnpiCommandError>(())
                    })?;
                    parameters
                } else {
                    println!("invalid response, command: {:?}", self);
                    return Err(UnpiCommandError::InvalidResponse);
                }
            }
        };
        Ok(parameters)
    }
}
