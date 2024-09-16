use crate::{
    command,
    parameters::ParameterType,
    utils::map::StaticMap,
    zstack::unpi::{
        commands::{Command, CommandBuffer},
        MessageType,
    },
};

command! {
    0,
    MessageType::AREQ,
    struct ResetRequest {
        type_: u8
    },
    struct ResetReqResponse {
    },
}

command! {
    1,
    MessageType::SREQ,
    struct PingRequest {
    },
    struct PingResponse {
        capabilities: u16
    },
}

command! {
    2,
    MessageType::SREQ,
    struct VersionRequest {
    },
    struct VersionResponse {
        transportrev: u8,
        product: u8,
        majorrel: u8,
        minorrel: u8,
        maintrel: u8,
        revision: u32
    },
}

command! {
    15,
    MessageType::SREQ,
    struct StackTuneRequest {
        operation: u8,
        value: i8
    },
    struct StackTuneResponse {
        value: u8
    },
}

command! {
    19,
    MessageType::SREQ,
    struct OsalNvLengthRequest {
        id: u16
    },
    struct OsalNvLengthResponse {
        length: u16
    },
}

command! {
    8,
    MessageType::SREQ,
    struct OsalNvReadRequest {
        id: u16,
        offset: u16
    },
    struct OsalNvReadResponse {
        status: u8,
        len: u8,
        value: CommandBuffer
    },
}

command! {
    28,
    MessageType::SREQ,
    struct OsalNvReadExtRequest {
        id: u16,
        offset: u16
    },
    struct OsalNvReadExtResponse {
        status: u8,
        len: u8,
        value: CommandBuffer
    },
}

command! {
    29,
    MessageType::SREQ,
    struct OsalNvWriteRequest {
        id: u16,
        offset: u16,
        len: u16,
        value: CommandBuffer
    },
    struct OsalNvWriteResponse {
        status: u8
    },
}
