use std::io::Read;

pub struct SliceReader<'a>(pub &'a [u8]);

#[allow(unused)]
impl<'a> SliceReader<'a> {
    pub fn read_i8(&mut self) -> Result<i8, std::io::Error> {
        let mut buffer = [0u8; 1];
        self.0.read_exact(&mut buffer[..])?;
        todo!("decide technique for converting u8 to i8");
        Ok(buffer[0] as i8)
    }

    pub fn read_u8(&mut self) -> Result<u8, std::io::Error> {
        let mut buffer = [0u8; 1];
        self.0.read_exact(&mut buffer[..])?;
        Ok(buffer[0])
    }

    pub fn read_u16_be(&mut self) -> Result<u16, std::io::Error> {
        let mut buffer = [0u8; 2];
        self.0.read_exact(&mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    pub fn read_u16_le(&mut self) -> Result<u16, std::io::Error> {
        let mut buffer = [0u8; 2];
        self.0.read_exact(&mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, std::io::Error> {
        let mut buffer = [0u8; 4];
        self.0.read_exact(&mut buffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    pub fn read_u8_array<const N: usize>(&mut self, len: usize) -> Result<[u8; N], std::io::Error> {
        let mut buffer = [0u8; N];
        self.0.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn read_u16_array<const N: usize>(
        &mut self,
        len: usize,
    ) -> Result<[u16; N], std::io::Error> {
        let mut buffer = [0u16; N];
        buffer.iter_mut().take(len).try_for_each(|x| {
            *x = self.read_u16_le()?;
            Ok::<(), std::io::Error>(())
        })?;
        Ok(buffer)
    }

    pub fn read_array_le<const N: usize>(&mut self, len: usize) -> Result<[u8; N], std::io::Error> {
        let mut buffer = [0u8; N];
        self.0.read_exact(&mut buffer)?;
        buffer.reverse();
        Ok(buffer)
    }

    pub fn read_exact(&mut self, output: &mut [u8]) -> Result<(), std::io::Error> {
        self.0.read_exact(output)
    }

    pub fn subslice_exact(&mut self, len: usize) -> Result<&'a [u8], std::io::Error> {
        let (left, right) = self.0.split_at_checked(len).ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "split at error",
        ))?;
        self.0 = right;
        Ok(left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x01, 0x02, 0x03];
        let mut reader = SliceReader(&data);
        assert_eq!(reader.read_u8().unwrap(), 0x01);
        assert_eq!(reader.read_u8().unwrap(), 0x02);
        assert_eq!(reader.read_u8().unwrap(), 0x03);
    }

    #[test]
    fn test_read_u16_le() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = SliceReader(&data);
        assert_eq!(reader.read_u16_le().unwrap(), 0x0201);
        assert_eq!(reader.read_u16_le().unwrap(), 0x0403);
    }

    #[test]
    fn test_read_u32_le() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut reader = SliceReader(&data);
        assert_eq!(reader.read_u32_le().unwrap(), 0x04030201);
        assert_eq!(reader.read_u32_le().unwrap(), 0x08070605);
    }

    #[test]
    fn test_read_array() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = SliceReader(&data);
        assert_eq!(
            reader.read_u8_array::<4>(4).unwrap(),
            [0x01, 0x02, 0x03, 0x04]
        );
    }

    #[test]
    fn test_read_array_le() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = SliceReader(&data);
        assert_eq!(
            reader.read_array_le::<4>(4).unwrap(),
            [0x04, 0x03, 0x02, 0x01]
        );
    }
}
