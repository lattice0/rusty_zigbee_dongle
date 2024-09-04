use crate::{
    unpi::{
        commands::{Command, ParameterType},
        MessageType,
    },
    utils::map::StaticMap,
};

pub const COMMANDS_ZDO: &[Command] = &[Command {
    name: "management_network_update_request",
    id: 55, // TODO: 0x0038 => 56?? (from zStackAdapter.ts)
    command_type: MessageType::SREQ,
    request: Some(StaticMap::new(&[
        ("dst_addr", ParameterType::U16),
        ("dst_addr_mode", ParameterType::U16),
        ("channel_mask", ParameterType::U32),
        ("scan_duration", ParameterType::U8),
        ("scan_count", ParameterType::U8),
        ("nwk_manager_addr", ParameterType::U16),
    ])),
    response: Some(StaticMap::new(&[("status", ParameterType::U8)])),
}];
