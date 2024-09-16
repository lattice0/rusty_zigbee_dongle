use crate::{
    parameters::ParameterType,
    utils::map::StaticMap,
    zstack::unpi::{commands::Command, MessageType},
};

pub const COMMANDS_UTIL: &[Command] = &[
    Command {
        name: "get_device_info",
        id: 0,
        command_type: MessageType::SREQ,
        request: None,
        response: Some(StaticMap::new(&[
            ("status", ParameterType::U8),
            ("ieee_addr", ParameterType::IeeeAddress),
            ("short_addr", ParameterType::U16),
            ("device_type", ParameterType::U8),
            ("device_state", ParameterType::U8),
            ("num_assoc_devices", ParameterType::U8),
            ("assoc_devices_list", ParameterType::ListU16(None)),
        ])),
    },
    Command {
        name: "led_control",
        id: 10,
        command_type: MessageType::SREQ,
        request: Some(StaticMap::new(&[
            ("led_id", ParameterType::U8),
            ("mode", ParameterType::U8),
        ])),
        response: Some(StaticMap::new(&[("status", ParameterType::U8)])),
    },
];
