use std::error::Error;

use bytebuffer::ByteBuffer;

pub trait ByteBufferExt {
    fn read_tri_byte(&mut self) -> Result<u32, Box<dyn Error>>;
}

impl ByteBufferExt for ByteBuffer {
    fn read_tri_byte(&mut self) -> Result<u32, Box<dyn Error>> {
        Ok(u32::from(self.read_u8()?) << 16
            | u32::from(self.read_u8()?) << 8
            | u32::from(self.read_u8()?))
    }
}
