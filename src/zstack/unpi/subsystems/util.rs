use deku::{ctx::BitSize, no_std_io, reader::Reader, DekuReader};
use deku::{prelude::*, DekuContainerRead, DekuWriter};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Serialize,
};

use crate::{
    command,
    utils::wrap_endianess::{WrapEndianess, WrapEndianessVisitor},
    zstack::unpi::{commands::List, MessageType, Subsystem},
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
        assoc_devices_list: List<u16>
    },
    NoDefaultSerialization
}

impl<'de> Deserialize<'de> for GetDeviceInfoResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(GetDeviceInfoResponseVisitor)
    }
}

struct GetDeviceInfoResponseVisitor;

impl<'de> Visitor<'de> for GetDeviceInfoResponseVisitor {
    type Value = GetDeviceInfoResponse;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a seq of 7 elements")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        println!("a");
        let status = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        println!("status: {:?}", status);
        println!("size hint: {:?}", seq.size_hint());
        println!("b");
        let ieee_addr = WrapEndianess::<'B', [u8; 8]>(seq.next_element::<[u8; 8]>()?.unwrap());
        let short_addr: u16 = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
        let short_addr = short_addr.to_le();
        println!("d");
        let device_type = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;
        println!("e");
        let device_state = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(4, &self))?;
        println!("f");
        let num_assoc_devices = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(5, &self))?;
        println!("g");
        let mut assoc_devices_list = List {
            list: [0u16; 255],
            len: num_assoc_devices as usize,
        };
        println!("num_assoc_devices: {:?}", num_assoc_devices);
        for _ in 0..(num_assoc_devices as usize) {
            let item: u16 = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(6, &self))?;
            assoc_devices_list.push(item.to_le());
        }
        println!("h");
        Ok(GetDeviceInfoResponse {
            status,
            ieee_addr: ieee_addr.0,
            short_addr: short_addr,
            device_type,
            device_state,
            num_assoc_devices,
            assoc_devices_list,
        })
    }
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

impl Serialize for GetDeviceInfoRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let seq = serializer.serialize_seq(Some(0))?;
        seq.end()
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
            <GetDeviceInfoResponse as DekuContainerRead>::from_reader((&mut cursor, 0)).unwrap();
        println!("{:?}", g);
    }
}
