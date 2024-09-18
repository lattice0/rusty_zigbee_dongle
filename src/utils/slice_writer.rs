use serde::{Deserializer, Serialize, Serializer};
use std::io::Write;

pub struct SliceWriter<'a>(pub &'a mut [u8]);

impl<'a> SliceWriter<'a> {
    pub fn new(slice: &'a mut [u8]) -> Self {
        SliceWriter(slice)
    }
}

impl<'a> SliceWriter<'a> {
    pub fn write_i8(&mut self, value: i8) -> Result<(), std::io::Error> {
        self.0.write_all(&[value as u8])?;
        Ok(())
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), std::io::Error> {
        self.0.write_all(&[value])?;
        Ok(())
    }

    pub fn write_u16_be(&mut self, value: u16) -> Result<(), std::io::Error> {
        self.0.write_all(&value.to_be_bytes())?;
        Ok(())
    }

    pub fn write_u16_le(&mut self, value: u16) -> Result<(), std::io::Error> {
        self.0.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    pub fn write_u32_le(&mut self, value: u32) -> Result<(), std::io::Error> {
        self.0.write_all(&value.to_le_bytes())?;
        Ok(())
    }

    pub fn write_u8_array<const N: usize>(&mut self, value: [u8; N]) -> Result<(), std::io::Error> {
        self.0.write_all(&value)?;
        Ok(())
    }

    pub fn write_u16_array<const N: usize>(
        &mut self,
        value: [u16; N],
    ) -> Result<(), std::io::Error> {
        value.iter().try_for_each(|x| self.write_u16_le(*x))?;
        Ok(())
    }
}

impl Serializer for SliceWriter<'_> {
    type Ok = ();

    type Error = SliceWriterError;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(mut self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.0.write_all(&[if v == true { 1 } else { 0 }])?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u8(mut self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.0.write_all(&[v])?;
        Ok(())
    }

    fn serialize_u16(mut self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.0.write_all(&v.to_be_bytes())?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub enum SliceWriterError {
    Io(std::io::Error),
}

impl serde::ser::Error for SliceWriterError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SliceWriterError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            msg.to_string(),
        ))
    }
}

impl serde::de::Error for SliceWriterError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        SliceWriterError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            msg.to_string(),
        ))
    }
}

impl std::fmt::Display for SliceWriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SliceWriterError {}

impl From<std::io::Error> for SliceWriterError {
    fn from(error: std::io::Error) -> Self {
        SliceWriterError::Io(error)
    }
}
