use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

#[derive(Debug, PartialEq, Clone)]
pub struct Buffer {
    pub buffer: [u8; 255],
    pub len: usize,
}

impl<'de> Deserialize<'de> for Buffer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Buffer", &["buffer", "len"], BufferVisitor())
    }
}

pub struct BufferVisitor();

impl<'de> Visitor<'de> for BufferVisitor {
    type Value = Buffer;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> {
        let mut buffer = [0; 255];
        buffer[0..v.len()].copy_from_slice(v);
        Ok(Buffer {
            buffer,
            len: v.len(),
        })
    }
}

impl Serialize for Buffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let slice = &self.buffer[0..self.len];
        serializer.serialize_bytes(slice)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_deserialize() {}
}
