use crate::{
    command,
    //utils::slice_reader::{ReadWithSliceReader, SliceReader},
    zstack::unpi::{commands::ListU16, MessageType, Subsystem},
};

command! {
    0,
    Subsystem::Util,
    MessageType::SREQ,
    struct GetDeviceInfoRequest {
    },
    struct GetDeviceInfoResponse {
        status: u8,
        ieee_addr: [u8; 8],
        short_addr: u16,
        device_type: u8,
        device_state: u8,
        num_assoc_devices: u8,
        assoc_devices_list: ListU16
    },
    NoDefaultSerialization
}

// impl<'a> ReadWithSliceReader for GetDeviceInfoResponse {
//     fn read_with_slice_reader<'b>(reader: SliceReader<'b>) -> Result<Self, std::io::Error> {
//         let mut reader = reader;
//         let status = reader.read_u8()?;
//         let ieee_addr: [u8; 8] = reader.read_u8_array(8)?;
//         let short_addr = reader.read_u16_le()?;
//         let device_type = reader.read_u8()?;
//         let device_state = reader.read_u8()?;
//         let num_assoc_devices = reader.read_u8()?;
//         let assoc_devices_list = reader.read_u16_array(num_assoc_devices as usize)?;
//         Ok(GetDeviceInfoResponse {
//             status,
//             ieee_addr,
//             short_addr,
//             device_type,
//             device_state,
//             num_assoc_devices,
//             assoc_devices_list: ListU16 {
//                 list: assoc_devices_list,
//                 len: num_assoc_devices as usize,
//             },
//         })
//     }
// }

command! {
    10,
    Subsystem::Util,
    MessageType::SREQ,
    struct LedControlRequest {
        led_id: u8,
         mode: u8
    },
    struct LedControlResponse {
        status: u8
    },
}
