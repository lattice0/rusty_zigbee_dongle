use crate::{
    command,
    zstack::unpi::{commands::List, MessageType, Subsystem},
};
use deku::{ctx::BitSize, no_std_io, reader::Reader, DekuReader};
use deku::{prelude::*, DekuContainerRead, DekuWriter};

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
        assoc_devices_list: List<u16>
    },
    NoDefaultSerialization
}

impl<'a> DekuReader<'a, ()> for GetDeviceInfoResponse {
    fn from_reader_with_ctx<R: no_std_io::Read + no_std_io::Seek>(
        reader: &mut Reader<R>,
        _ctx: (),
    ) -> Result<Self, deku::DekuError> {
        let status = u8::from_reader_with_ctx(reader, BitSize::of::<u8>())?;
        let ieee_addr = <[u8; 8]>::from_reader_with_ctx(reader, BitSize::of::<[u8; 8]>())?;
        let short_addr = u16::from_reader_with_ctx(reader, BitSize::of::<u16>())?;
        let device_type = u8::from_reader_with_ctx(reader, BitSize::of::<u8>())?;
        let device_state = u8::from_reader_with_ctx(reader, BitSize::of::<u8>())?;
        let num_assoc_devices = u8::from_reader_with_ctx(reader, BitSize::of::<u8>())?;
        let mut assoc_devices_list = List {
            list: [0u16; 255],
            len: num_assoc_devices as usize,
        };
        for _ in 0..(num_assoc_devices as usize) {
            let item = u16::from_reader_with_ctx(reader, BitSize::of::<u16>())?;
            assoc_devices_list.push(item);
        }
        Ok(GetDeviceInfoResponse {
            status,
            ieee_addr,
            short_addr,
            device_type,
            device_state,
            num_assoc_devices,
            assoc_devices_list,
        })
    }
}

impl<'a> DekuContainerRead<'a> for GetDeviceInfoResponse {
    fn from_reader<R: no_std_io::Read + no_std_io::Seek>(
        input: (&'a mut R, usize),
    ) -> Result<(usize, Self), DekuError>
    where
        Self: Sized,
    {
        let reader = &mut deku::reader::Reader::new(input.0);
        if input.1 != 0 {
            reader.skip_bits(input.1)?;
        }

        let value = Self::from_reader_with_ctx(reader, ())?;

        Ok((reader.bits_read, value))
    }

    fn from_bytes(input: (&'a [u8], usize)) -> Result<((&'a [u8], usize), Self), DekuError>
    where
        Self: Sized,
    {
        let mut cursor = no_std_io::Cursor::new(input.0);
        let reader = &mut deku::reader::Reader::new(&mut cursor);
        if input.1 != 0 {
            reader.skip_bits(input.1)?;
        }

        let __deku_value = Self::from_reader_with_ctx(reader, ())?;
        let read_whole_byte = (reader.bits_read % 8) == 0;
        let idx = if read_whole_byte {
            reader.bits_read / 8
        } else {
            (reader.bits_read - (reader.bits_read % 8)) / 8
        };
        Ok(((&input.0[idx..], reader.bits_read % 8), __deku_value))
    }
}

impl DekuWriter<()> for GetDeviceInfoRequest {
    fn to_writer<W: no_std_io::Write + no_std_io::Seek>(
        &self,
        writer: &mut deku::writer::Writer<W>,
        _ctx: (),
    ) -> Result<(), deku::DekuError> {
        Ok(())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::slice_reader::SliceReader;
    use deku::ctx::BitSize;
    use deku::writer::Writer;
    use deku::{prelude::*, DekuWriter};
    use deku::{DekuContainerRead, DekuRead};
    //use no_std_io::io::{Seek, Write};

    #[test]
    fn test_get_device_info() {
        let data = [0, 175, 60, 67, 1, 0, 75, 18, 0, 0, 0, 7, 9, 0];
        let mut reader = SliceReader(&data);
        let mut cursor = std::io::Cursor::new(&data);
        let g =
            GetDeviceInfoResponse::from_reader((&mut cursor, 0)).unwrap();
        println!("{:?}", g);
    }
}
