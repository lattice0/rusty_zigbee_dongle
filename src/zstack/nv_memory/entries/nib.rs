use deku::DekuRead;

//TODO: non_snake_case because of deku. How to remove/fix?
#[allow(non_snake_case)]
#[derive(Debug, DekuRead)]
pub struct Nib {
    pub sequence_num: u8,
    pub passive_ack_timeout: u8,
    pub max_broadcast_retries: u8,
    pub max_children: u8,
    pub max_depth: u8,
    pub max_routers: u8,
    pub dummy_neighbor_table: u8,
    pub broadcast_delivery_time: u8,
    pub report_constant_cost: u8,
    pub route_disc_retries: u8,
    pub dummy_routing_table: u8,
    pub secure_all_frames: u8,
    pub security_level: u8,
    pub sym_link: u8,
    pub capability_flags: u8,
    pub transaction_persistence_time: u16,
    pub nwk_protocol_version: u8,
    pub route_discovery_time: u8,
    pub route_expiry_time: u8,
    pub nwk_dev_address: u16,
    pub nwk_logical_channel: u8,
    pub nwk_coord_address: u16,
    #[deku(endian = "little")]
    pub nwk_coord_ext_address: [u8; 8],
    pub nwk_pan_id: u16,
    pub nwk_state: u8,
    pub channel_list: u32,
    pub beacon_order: u8,
    pub super_frame_order: u8,
    pub scan_duration: u8,
    pub batt_life_ext: u8,
    pub allocated_router_addresses: u32,
    pub allocated_end_device_addresses: u32,
    pub node_depth: u8,
    #[deku(endian = "little")]
    pub extended_panid: [u8; 8],
    pub nwk_key_loaded: u8,
    pub spare1: NwkKeyDescriptor,
    pub spare2: NwkKeyDescriptor,
    pub spare3: u8,
    pub spare4: u8,
    pub nwk_link_status_period: u8,
    pub nwk_router_age_limit: u8,
    pub nwk_use_multi_cast: u8,
    pub nwk_is_concentrator: u8,
    pub nwk_concentrator_discovery_time: u8,
    pub nwk_concentrator_radius: u8,
    pub nwk_all_fresh: u8,
    pub nwk_manager_addr: u16,
    pub nwk_total_transmissions: u16,
    pub nwk_update_id: u8,
}

//TODO: non_snake_case because of deku. How to remove/fix?
#[allow(non_snake_case)]
#[derive(Debug, DekuRead)]
pub struct NwkKeyDescriptor {
    pub key_seq_num: u8,
    pub key: [u8; 16],
}
