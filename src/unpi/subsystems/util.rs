use crate::{
    unpi::{
        commands::{Command, ParameterType},
        MessageType,
    },
    utils::map::StaticMap,
};

pub const COMMANDS_UTIL: &[Command] = &[Command {
    name: "led_control",
    id: 10,
    command_type: MessageType::SREQ,
    request: Some(StaticMap::new(&[("led_id", ParameterType::U8), ("mode", ParameterType::U8)])),
    response: Some(StaticMap::new(&[("status", ParameterType::U8)])),
}];
