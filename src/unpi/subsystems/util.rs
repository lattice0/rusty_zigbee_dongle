use crate::{
    unpi::{
        commands::{Command, ParameterType},
        MessageType,
    },
    utils::Map,
};

pub const COMMANDS_UTIL: &[Command] = &[Command {
    name: "led_control",
    id: 10,
    command_type: MessageType::SREQ,
    request: Map::new(&[("led_id", ParameterType::U8), ("mode", ParameterType::U8)]),
    response: Map::new(&[("status", ParameterType::U8)]),
}];
