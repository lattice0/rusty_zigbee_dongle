use serde::{de::Visitor, Deserialize};

/// Wraps an array and gives you the opportunity to specify the endieness
/// ENDIANESS = 'B' for big endian, 'L' for little endiang
#[derive(Debug)]
pub struct WrapEndianess<const ENDIANESS: char, T>(pub T);

impl<'de, const ENDIANESS: char, const N: usize, T: Default + Copy> Deserialize<'de>
    for WrapEndianess<ENDIANESS, [T; N]>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(deserializer.deserialize_seq(WrapEndianessVisitor::<ENDIANESS, [T; N]>::new())?)
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

impl<'a, const ENDIANESS: char, const N: usize, T: Default + Copy + Deserialize<'a>> Visitor<'a>
    for WrapEndianessVisitor<ENDIANESS, [T; N]>
{
    type Value = WrapEndianess<ENDIANESS, [T; N]>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an array of 8 bytes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'a>,
    {
        let mut value = [T::default(); N];
        for i in 0..N {
            value[i] = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
        }
        if ENDIANESS == 'L' {
            value.reverse();
        }
        Ok(WrapEndianess(value))
    }
}
