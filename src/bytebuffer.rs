use bytebuffer::ByteBuffer;
use std::error::Error;

pub trait ByteBufferExt {
    fn read_tri_byte(&mut self) -> Result<u32, Box<dyn Error>>;
}
impl ByteBufferExt for ByteBuffer {
    fn read_tri_byte(&mut self) -> Result<u32, Box<dyn Error>> {
       unimplemented!("reading tri-byte is not implemented!")
    }
}
