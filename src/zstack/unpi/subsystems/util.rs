use crate::{
    command,
    parameters::ParameterType,
    utils::map::StaticMap,
    zstack::unpi::{
        commands::{Command, CommandListU16},
        MessageType,
    },
};

command! {
    0,
    MessageType::SREQ,
    struct GetDeviceInfoRequest {
    },
    struct GetDeviceInfoResponse {
        status: u8,
        ieee_addr: u64,
        short_addr: u16,
        device_type: u8,
        device_state: u8,
        num_assoc_devices: u8,
        assoc_devices_list: CommandListU16
    },
}

command! {
    10,
    MessageType::SREQ,
    struct LedControlRequest {
        led_id: u8,
        mode: u8
    },
    struct LedControlResponse {
        status: u8
    },
}
