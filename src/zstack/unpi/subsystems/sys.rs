use crate::{
    command, coordinator::ResetType, zstack::unpi::{
        buffer::Buffer,
        MessageType, Subsystem,
    }
};


command! {
    0,
    Subsystem::Sys,
    MessageType::AREQ,
    struct ResetRequest {
        reset_type: ResetType
    },
    struct ResetReqResponse {
    },
}

command! {
    1,
    Subsystem::Sys,
    MessageType::SREQ,
    struct PingRequest {
    },
    struct PingResponse {
        capabilities: u16
    },
}

command! {
    2,
    Subsystem::Sys,
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
    Subsystem::Sys,
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
    Subsystem::Sys,
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
    Subsystem::Sys,
    MessageType::SREQ,
    struct OsalNvReadRequest {
        id: u16,
        offset: u16
    },
    struct OsalNvReadResponse {
        status: u8,
        len: u8,
        value: Buffer
    },
}

command! {
    28,
    Subsystem::Sys,
    MessageType::SREQ,
    struct OsalNvReadExtRequest {
        id: u16,
        offset: u16
    },
    struct OsalNvReadExtResponse {
        status: u8,
        len: u8,
        value: Buffer
    },
}

command! {
    29,
    Subsystem::Sys,
    MessageType::SREQ,
    struct OsalNvWriteRequest {
        id: u16,
        offset: u16,
        len: u16,
        value: Buffer
    },
    struct OsalNvWriteResponse {
        status: u8
    },
}
