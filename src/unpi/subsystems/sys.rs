use crate::{
    unpi::{
        commands::{Command, ParameterType},
        MessageType,
    },
    utils::Map,
};

pub const COMMANDS_SYS: &[Command] = &[Command {
    name: "stack_tune",
    id: 15,
    command_type: MessageType::SREQ,
    request: Map::new(&[
        ("operation", ParameterType::U8),
        ("value", ParameterType::I8),
    ]),
    response: Map::new(&[("value", ParameterType::U8)]),
}];