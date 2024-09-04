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
