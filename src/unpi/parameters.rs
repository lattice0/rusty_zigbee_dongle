use super::commands::ParameterError;
use crate::utils::slice_reader::SliceReader;
use std::io::Write;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ParameterType {
    U8,
    U16,
    U32,
    I8,
    IeeeAddress,
    ListU16(Option<usize>),
}

impl ParameterType {
    pub fn from_slice_reader(
        &self,
        reader: &mut SliceReader,
    ) -> Result<ParameterValue, ParameterError> {
        Ok(match self {
            ParameterType::U8 => ParameterValue::U8(reader.read_u8()?),
            ParameterType::U16 => ParameterValue::U16(reader.read_u16_le()?),
            ParameterType::U32 => ParameterValue::U32(reader.read_u32_le()?),
            ParameterType::I8 => ParameterValue::I8(reader.read_i8()?),
            ParameterType::IeeeAddress => ParameterValue::IeeAddress(reader.read_u8_array(8)?),
            ParameterType::ListU16(len) => ParameterValue::ListU16(
                reader.read_u16_array(len.ok_or(ParameterError::MissingListLength)?)?,
            ),
        })
    }
}

impl ParameterValue {
    pub fn try_into_u8(&self) -> Result<u8, ParameterError> {
        match self {
            ParameterValue::U8(v) => Ok(*v),
            _ => Err(ParameterError::InvalidParameter),
        }
    }

    pub fn try_into_u16(&self) -> Result<u16, ParameterError> {
        match self {
            ParameterValue::U16(v) => Ok(*v),
            _ => Err(ParameterError::InvalidParameter),
        }
    }

    pub fn try_into_u32(&self) -> Result<u32, ParameterError> {
        match self {
            ParameterValue::U32(v) => Ok(*v),
            _ => Err(ParameterError::InvalidParameter),
        }
    }

    pub fn try_into_i8(&self) -> Result<i8, ParameterError> {
        match self {
            ParameterValue::I8(v) => Ok(*v),
            _ => Err(ParameterError::InvalidParameter),
        }
    }

    pub fn try_into_ieee_addr(&self) -> Result<[u8; 8], ParameterError> {
        match self {
            ParameterValue::IeeAddress(v) => Ok(*v),
            _ => Err(ParameterError::InvalidParameter),
        }
    }

    pub fn try_into_list_u16(&self) -> Result<[u16; 16], ParameterError> {
        match self {
            ParameterValue::ListU16(v) => Ok(*v),
            _ => Err(ParameterError::InvalidParameter),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParameterValue {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    IeeAddress([u8; 8]),
    ListU16([u16; 16]),
}

impl PartialEq<ParameterType> for ParameterValue {
    fn eq(&self, other: &ParameterType) -> bool {
        match self {
            ParameterValue::U8(_) => other == &ParameterType::U8,
            ParameterValue::U16(_) => other == &ParameterType::U16,
            ParameterValue::U32(_) => other == &ParameterType::U32,
            ParameterValue::I8(_) => other == &ParameterType::I8,
            ParameterValue::IeeAddress(_) => other == &ParameterType::IeeeAddress,
            ParameterValue::ListU16(_) => matches!(other, ParameterType::ListU16(_)),
        }
    }
}

impl ParameterValue {
    pub fn match_and_write(
        &self,
        parameter_type: &ParameterType,
        mut output: &mut [u8],
    ) -> Result<usize, ParameterError> {
        let len = output.len();
        if self != parameter_type {
            return Err(ParameterError::InvalidParameter);
        }
        match self {
            ParameterValue::U8(v) => output.write_all(&[*v])?,
            ParameterValue::U16(v) => output.write_all(&v.to_le_bytes())?,
            ParameterValue::U32(v) => output.write_all(&v.to_le_bytes())?,
            //TODO: i8 to u8?
            ParameterValue::I8(v) => output.write_all(&[*v as u8])?,
            ParameterValue::IeeAddress(v) => output.write_all(v)?,
            ParameterValue::ListU16(v) => {
                output.write_all(&(v.len() as u8).to_le_bytes())?;
                for i in v {
                    output.write_all(&i.to_le_bytes())?;
                }
            }
        }
        Ok(len - output.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::slice_reader::SliceReader;

    #[test]
    fn read_u8() {
        let mut reader = SliceReader(&[0x01]);
        let value = reader.read_u8().unwrap();
        assert_eq!(value, 0x01);
    }

    #[test]
    fn read_ieee_address() {
        let mut reader = SliceReader(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
        let value = reader.read_u8_array(6).unwrap();
        assert_eq!(value, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }
}
