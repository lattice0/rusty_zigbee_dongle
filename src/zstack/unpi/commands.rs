use super::{MessageType, Subsystem};
use crate::{
    parameters::{ParameterType, ParameterValue},
    utils::map::StaticMap,
};
use serde::{Deserialize, Serialize};

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

pub trait CommandRequest: std::fmt::Debug {
    type Response: CommandResponse;

    fn id() -> u8;
    fn message_type() -> MessageType;
    fn subsystem() -> Subsystem;

    fn self_id(&self) -> u8;
    fn self_message_type(&self) -> MessageType;
    fn self_subsystem(&self) -> Subsystem;
}

pub trait CommandResponse {
    fn id() -> u8;
    fn message_type() -> MessageType;
    fn subsystem() -> Subsystem;
    fn self_id(&self) -> u8;
    fn self_message_type(&self) -> MessageType;
    fn self_subsystem(&self) -> Subsystem;
}

#[derive(Debug, PartialEq, Clone)]
pub struct ListU16 {
    pub list: [u16; 255],
    pub len: usize,
}

//TODO alloc only
impl Serialize for ListU16 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.list[..self.len].to_vec().serialize(serializer)
    }
}

//TODO alloc only
impl<'de> Deserialize<'de> for ListU16 {
    fn deserialize<D>(deserializer: D) -> Result<ListU16, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let list: Vec<u16> = Vec::deserialize(deserializer)?;
        let mut arr = [0; 255];
        for (i, v) in list.iter().enumerate() {
            arr[i] = *v;
        }
        Ok(ListU16 {
            list: arr,
            len: list.len(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CommandIeeeAddress {
    pub ieee_address: [u8; 8],
}

#[macro_export]
macro_rules! command {
    // Match a struct declaration with fields
    (
     // command ID
     $id: literal,
     // subsystem type (Subsystem::SYS, Subsystem::MAC etc)
     $sty: expr,
     // message type (MessageType::SREQ, MessageType::AREQ, MessageType::SRSP etc)
     $mty: expr,
     // Request struct with fields
     struct $name:ident  { $($field:ident : $type:ty ),* },
     // Response struct with fields
     struct $rname:ident { $($rfield:ident : $rtype:ty ),* },
    ) => {
        command!(
            $id,
            $sty,
            $mty,
            struct $name { $( $field : $type ),* },
            struct $rname { $( $rfield : $rtype ),* },
            WithDefaultSerialization
        );

    };
    (
        $id: literal,
        $sty: expr,
        $mty: expr,
        struct $name:ident { $( $field:ident : $type:ty ),* },
        struct $rname:ident { $( $rfield:ident : $rtype:ty ),* },
        // TryFrom<SliceReader> custom implementation that overrides the default one
        WithDefaultSerialization
    ) => {
        command!(
            $id,
            $sty,
            $mty,
            struct $name { $( $field : $type ),* },
            struct $rname { $( $rfield : $rtype ),* },
            NoDefaultSerialization
        );
        // Default reader implementation here
    };
    (
        $id: literal,
        $sty: expr,
        $mty: expr,
        struct $name:ident { $( $field:ident : $type:ty ),* },
        struct $rname:ident { $( $rfield:ident : $rtype:ty ),* },
        // TryFrom<SliceReader> custom implementation that overrides the default one
        NoDefaultSerialization
    ) => {
        #[allow(dead_code)]
        #[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            $(pub $field: $type ),*
        }

        impl $crate::zstack::unpi::commands::CommandRequest for $name {
            type Response = $rname;

            fn id() -> u8 {
                $id
            }

            fn message_type() -> $crate::zstack::unpi::MessageType {
                $mty
            }

            fn subsystem() -> $crate::zstack::unpi::Subsystem {
                $sty
            }

            fn self_id(&self) -> u8 {
                Self::id()
            }

            fn self_message_type(&self) -> $crate::zstack::unpi::MessageType {
                Self::message_type()
            }

            fn self_subsystem(&self) -> $crate::zstack::unpi::Subsystem {
                Self::subsystem()
            }
        }

        #[allow(dead_code)]
        #[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $rname {
            $(pub $rfield: $rtype ,)*
        }

        impl $crate::zstack::unpi::commands::CommandResponse for $rname {
            fn message_type() -> $crate::zstack::unpi::MessageType {
                match $mty {
                    $crate::zstack::unpi::MessageType::SREQ => $crate::zstack::unpi::MessageType::SRESP,
                    $crate::zstack::unpi::MessageType::AREQ => $crate::zstack::unpi::MessageType::SREQ,
                    _ => $mty
                }
            }

            fn id() -> u8 {
                $id
            }

            fn subsystem() -> $crate::zstack::unpi::Subsystem {
                $sty
            }

            fn self_id(&self) -> u8 {
                Self::id()
            }

            fn self_message_type(&self) -> $crate::zstack::unpi::MessageType {
                Self::message_type()
            }

            fn self_subsystem(&self) -> $crate::zstack::unpi::Subsystem {
                Self::subsystem()
            }
        }

        $crate::impl_status_if_has_status_field! { struct $rname { $( $rfield : $rtype ),* } }
    };
}

// If the response struct has a field named `status`, implement TryInto<CommandStatus> for it
#[macro_export]
macro_rules! impl_status_if_has_status_field {
    (
        struct $name:ident {
            status: $status_type:ty $(, $($rest:tt)*)?
        }
    ) => {
        #[allow(dead_code)]
        impl TryInto<$crate::zstack::unpi::constants::CommandStatus> for $name {
            type Error = $crate::zstack::unpi::constants::NoCommandStatusError;

            fn try_into(
                self,
            ) -> Result<$crate::zstack::unpi::constants::CommandStatus, Self::Error> {
                Ok(
                    $crate::zstack::unpi::constants::CommandStatus::try_from(self.status)
                        .map_err(|_| $crate::zstack::unpi::constants::NoCommandStatusError)?,
                )
            }
        }
    };
    (
        struct $name:ident {
            $($field:ident : $type:ty),*
        }
    ) => {};
}

pub trait IntoBytes {
    type Output;
    fn into_bytes(output: &mut [u8]) -> Self::Output;
}

#[derive(Debug)]
pub enum IntoBytesError {
    InvalidParameter,
    InvalidLength,
}
