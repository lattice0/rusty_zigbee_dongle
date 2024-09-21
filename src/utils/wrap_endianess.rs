use crate::utils::slice_reader::SliceReader;
use serde::{
    de::{SeqAccess, Visitor},
    Deserialize,
};

/// Wraps an array and gives you the opportunity to specify the endieness
/// ENDIANESS = 'B' for big endian, 'L' for little endiang
#[derive(Debug)]
pub struct WrapEndianess<const ENDIANESS: char, T>(pub T);

impl<'de, const ENDIANESS: char, const N: usize> Deserialize<'de>
    for WrapEndianess<ENDIANESS, [u8; N]>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(N, WrapEndianessVisitor::new())
    }
}

impl<'de> Deserialize<'de> for WrapEndianess<'E', u16> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(WrapEndianess(u16::deserialize(deserializer)?))
    }
}

impl<'de> Deserialize<'de> for WrapEndianess<'L', u16> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(WrapEndianess(u16::deserialize(deserializer)?.to_le()))
    }
}

pub struct WrapEndianessVisitor<const ENDIANESS: char, T> {
    marker: std::marker::PhantomData<WrapEndianess<ENDIANESS, T>>,
}

impl<const ENDIANESS: char, T> WrapEndianessVisitor<ENDIANESS, T> {
    pub fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

impl<const ENDIANESS: char, T> Default for WrapEndianessVisitor<ENDIANESS, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const ENDIANESS: char, const N: usize> Visitor<'a>
    for WrapEndianessVisitor<ENDIANESS, [u8; N]>
{
    type Value = WrapEndianess<ENDIANESS, [u8; N]>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an array of 8 bytes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        println!("visit_seq");
        let mut value = [0u8; N];
        for (i, item) in value.iter_mut().enumerate().take(N) {
            *item = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
        }
        if ENDIANESS == 'L' {
            value.reverse();
        }
        Ok(WrapEndianess(value))
    }

    // fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    // where
    //     E: serde::de::Error,
    // {
    //     println!("visit_borrowed_bytes");
    //     let mut value = [0u8; N];
    //     value[..N].copy_from_slice(v);
    //     // let mut slice_reader = SliceReader(v);
    //     // for (_, item) in value.iter_mut().enumerate().take(N) {
    //     //     *item = slice_reader
    //     //         .read_u8()
    //     //         .map_err(|e| serde::de::Error::custom(e))?;
    //     // }
    //     if ENDIANESS == 'L' {
    //         value.reverse();
    //     }
    //     Ok(WrapEndianess(value))
    // }
}

#[cfg(test)]
mod tests {
    use super::WrapEndianessVisitor;
    use serde::de::Visitor;

    #[test]
    fn test_wrap_endianess() {
        let data = [0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let data_le = [0x08u8, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01];
        let wrapped = WrapEndianessVisitor::<'B', [u8; 8]>::new()
            .visit_borrowed_bytes::<serde::de::value::Error>(&data)
            .unwrap();
        assert_eq!(data, wrapped.0);
        let wrapped = WrapEndianessVisitor::<'L', [u8; 8]>::new()
            .visit_borrowed_bytes::<serde::de::value::Error>(&data)
            .unwrap();
        assert_eq!(data_le, wrapped.0);
    }
}
