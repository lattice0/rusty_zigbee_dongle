use deku::{reader::Reader, writer::Writer, DekuError, DekuReader, DekuWriter};
use std::io::{Read, Seek, Write};

#[derive(Debug, PartialEq, Clone)]
pub struct Buffer {
    pub buffer: [u8; 255],
    pub len: usize,
}

impl DekuWriter<()> for Buffer {
    #[doc = " Write type to bytes"]
    fn to_writer<W: Write + Seek>(
        &self,
        writer: &mut Writer<W>,
        _ctx: (),
    ) -> Result<(), DekuError> {
        writer.write_bytes(&self.buffer[..self.len])?;
        Ok(())
    }
}

impl<'a> DekuReader<'a, ()> for Buffer {
    fn from_reader_with_ctx<R: Read + Seek>(
        reader: &mut Reader<R>,
        _ctx: (),
    ) -> Result<Self, DekuError>
    where
        Self: Sized,
    {
        let mut output = [0u8; 255];
        let r = reader.read_bytes(output.len(), &mut output)?;
        Ok(Buffer {
            buffer: output,
            len: output.len(),
        })
    }
}
