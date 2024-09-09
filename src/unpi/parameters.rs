use super::commands::ParameterError;
use crate::utils::slice_reader::SliceReader;
use std::io::Write;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ParameterType {
    U8,
    U16,
    U32,
    I8,
    IdeeAddr,
    ListU16,
}

impl ParameterType {
    pub fn from_slice_reader(
        &self,
        reader: &mut SliceReader,
    ) -> Result<ParameterValue, ParameterError> {
        /*
        const octets = Array.from(this.readBuffer(8).reverse());
        return `0x${octets.map((octet) => octet.toString(16).padStart(2, '0')).join('')}`;
         */
        Ok(match self {
            ParameterType::U8 => ParameterValue::U8(reader.read_u8()?),
            ParameterType::U16 => ParameterValue::U16(reader.read_u16_le()?),
            ParameterType::U32 => ParameterValue::U32(reader.read_u32_le()?),
            ParameterType::I8 => ParameterValue::I8(reader.read_i8()?),
            ParameterType::IdeeAddr => ParameterValue::IeeAddress(reader.read_array(8)?),
            ParameterType::ListU16 => todo!(),
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParameterValue {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    IeeAddress([u8; 8]),
    ListU16([u8; 16]),
}

impl PartialEq<ParameterType> for ParameterValue {
    fn eq(&self, other: &ParameterType) -> bool {
        match self {
            ParameterValue::U8(_) => other == &ParameterType::U8,
            ParameterValue::U16(_) => other == &ParameterType::U16,
            ParameterValue::U32(_) => other == &ParameterType::U32,
            ParameterValue::I8(_) => other == &ParameterType::I8,
            ParameterValue::IeeAddress(_) => other == &ParameterType::IdeeAddr,
            ParameterValue::ListU16(_) => other == &ParameterType::ListU16,
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
mod tests {}
