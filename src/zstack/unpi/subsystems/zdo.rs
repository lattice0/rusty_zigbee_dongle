use crate::{
    command,
    zstack::unpi::{commands::CommandIeeeAddress, MessageType, Subsystem},
};

command! {
    54,
    Subsystem::Zdo,
    MessageType::SREQ,
    struct ManagementPermitJoinRequest {
        address_mode: u16,
        destination_address: u16,
        duration: u8,
        tc_significance: u8
    },
    struct ManagementPermitJoinResponse {
        status: u8
    },
}

command! {
    55,
    Subsystem::Zdo,
    MessageType::SREQ,
    struct ManagementNetworkUpdateRequest {
        destination_address: u16,
        destination_address_mode: u16,
        channel_mask: u32,
        scan_duration: u8,
        scan_count: u8,
        network_manager_address: u16
    },
    struct ManagementNetworkUpdateResponse {
        status: u8
    },
}

command! {
    64,
    Subsystem::Zdo,
    MessageType::SREQ,
    struct StartupFromAppRequest {
        start_delay: u16,
        status: u8
    },
    struct StartupFromAppResponse {
        status: u8
    },
}

command! {
    69,
    Subsystem::Zdo,
    MessageType::SREQ,
    struct ExitRouteDiscRequest {
        destination_address: u16,
        options: u8,
        radius: u8
    },
    struct ExitRouteDiscResponse {
        status: u8
    },
}

command! {
    192,
    Subsystem::Zdo,
    MessageType::AREQ,
    struct StateChangedIndRequest {
        state: u8
    },
    struct StateChangedIndResponse {

    },
}

command! {
    202,
    Subsystem::Zdo,
    MessageType::AREQ,
    struct TcDeviceIndexRequest {
        network_address: u16,
        extended_address: CommandIeeeAddress,
        parent_address: u16
    },
    struct TcDeviceIndexResponse {

    },
}
