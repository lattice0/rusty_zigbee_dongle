use crate::{
    unpi::{
        commands::{Command, ParameterType},
        MessageType,
    },
    utils::map::StaticMap,
};

pub const COMMANDS_SYS: &[Command] = &[
    Command {
        name: "reset_req",
        id: 0,
        command_type: MessageType::AREQ,
        request: Some(StaticMap::new(&[("type", ParameterType::U8)])),
        response: None,
    },
    Command {
        name: "ping",
        id: 1,
        command_type: MessageType::SREQ,
        request: None,
        response: Some(StaticMap::new(&[("capabilities", ParameterType::U16)])),
    },
    Command {
        name: "version",
        id: 2,
        command_type: MessageType::SREQ,
        request: None,
        response: Some(StaticMap::new(&[
            ("transportrev", ParameterType::U8),
            ("product", ParameterType::U8),
            ("majorrel", ParameterType::U8),
            ("minorrel", ParameterType::U8),
            ("maintrel", ParameterType::U8),
            ("revision", ParameterType::U32),
        ])),
    },
    Command {
        name: "stack_tune",
        id: 15,
        command_type: MessageType::SREQ,
        request: Some(StaticMap::new(&[
            ("operation", ParameterType::U8),
            ("value", ParameterType::I8),
        ])),
        response: Some(StaticMap::new(&[("value", ParameterType::U8)])),
    },
];
