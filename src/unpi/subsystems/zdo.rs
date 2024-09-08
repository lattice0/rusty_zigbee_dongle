use crate::{
    unpi::{
        commands::{Command, ParameterType},
        MessageType,
    },
    utils::map::StaticMap,
};

pub const COMMANDS_ZDO: &[Command] = &[
    Command {
        name: "management_permit_join_request",
        id: 54,
        command_type: MessageType::SREQ,
        request: Some(StaticMap::new(&[
            ("address_mode", ParameterType::U16),
            ("destination_address", ParameterType::U16),
            ("duration", ParameterType::U8),
            ("tc_significance", ParameterType::U8),
        ])),
        response: Some(StaticMap::new(&[("status", ParameterType::U8)])),
    },
    Command {
        name: "management_network_update_request",
        id: 55, // TODO: 0x0038 => 56?? (from zStackAdapter.ts)
        command_type: MessageType::SREQ,
        request: Some(StaticMap::new(&[
            ("destination_address", ParameterType::U16),
            ("destination_address_mode", ParameterType::U16),
            ("channel_mask", ParameterType::U32),
            ("scan_duration", ParameterType::U8),
            ("scan_count", ParameterType::U8),
            ("network_manager_address", ParameterType::U16),
        ])),
        response: Some(StaticMap::new(&[("status", ParameterType::U8)])),
    },
    Command {
        name: "exit_route_disc",
        id: 69, // TODO: 0x0038 => 56?? (from zStackAdapter.ts)
        command_type: MessageType::SREQ,
        request: Some(StaticMap::new(&[
            ("destination_address", ParameterType::U16),
            ("options", ParameterType::U8),
            ("radius", ParameterType::U8),
        ])),
        response: Some(StaticMap::new(&[("status", ParameterType::U8)])),
    },
];
