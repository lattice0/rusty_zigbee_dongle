use super::{MessageType, Subsystem};
use deku::{DekuRead, DekuWrite};

pub const MAX_COMMAND_SIZE: usize = 15;

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
pub struct List<T> {
    pub list: [T; 255],
    pub len: usize,
}

impl<T: Copy + Default> List<T> {
    pub fn new() -> Self {
        List {
            list: [Default::default(); 255],
            len: 0,
        }
    }

    pub fn push(&mut self, item: T) {
        self.list[self.len] = item;
        self.len += 1;
    }
}


#[derive(Debug, PartialEq, Clone, DekuRead, DekuWrite)]
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
        #[allow(dead_code)]
        #[derive(Debug, PartialEq, Clone, deku::DekuRead, deku::DekuWrite)]
        pub struct $name {
            $(pub $field: $type ),*
        }

        #[allow(dead_code)]
        #[derive(Debug, PartialEq, Clone, deku::DekuRead, deku::DekuWrite)]
        pub struct $rname {
            $(pub $rfield: $rtype ,)*
        }

        command!(
            $id,
            $sty,
            $mty,
            struct $name { $( $field : $type ),* },
            struct $rname { $( $rfield : $rtype ),* },
            Final
        );
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
        #[derive(Debug, PartialEq, Clone)]
        pub struct $name {
            $(pub $field: $type ),*
        }

        #[allow(dead_code)]
        #[derive(Debug, PartialEq, Clone)]
        pub struct $rname {
            $(pub $rfield: $rtype ,)*
        }

        command!(
            $id,
            $sty,
            $mty,
            struct $name { $( $field : $type ),* },
            struct $rname { $( $rfield : $rtype ),* },
            Final
        );
    };
    (
        $id: literal,
        $sty: expr,
        $mty: expr,
        struct $name:ident { $( $field:ident : $type:ty ),* },
        struct $rname:ident { $( $rfield:ident : $rtype:ty ),* },
        // TryFrom<SliceReader> custom implementation that overrides the default one
        Final
    ) => {
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
                $crate::zstack::unpi::constants::CommandStatus::try_from(self.status)
                    .map_err(|_| $crate::zstack::unpi::constants::NoCommandStatusError)
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
