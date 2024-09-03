use crate::{
    unpi::{
        commands::{Command, ParameterType},
        MessageType,
    },
    utils::Map,
};

pub const COMMANDS_SYS: &[Command] = &[
    Command {
        name: "stack_tune",
        id: 15,
        command_type: MessageType::SREQ,
        request: Some(Map::new(&[
            ("operation", ParameterType::U8),
            ("value", ParameterType::I8),
        ])),
        response: Some(Map::new(&[("value", ParameterType::U8)])),
    },
    Command {
        name: "reset_req",
        id: 0,
        command_type: MessageType::AREQ,
        request: Some(Map::new(&[("type", ParameterType::U8)])),
        response: None,
    },
    Command {
        name: "ping",
        id: 1,
        command_type: MessageType::SREQ,
        request: None,
        response: Some(Map::new(&[("value", ParameterType::U8)])),
    },
];
