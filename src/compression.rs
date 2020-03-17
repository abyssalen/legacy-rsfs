use std::error::Error;
use std::io::{Read, Write};

use bytebuffer::ByteBuffer;
use bzip2::read::BzDecoder;
use libflate::gzip::{Decoder, Encoder, HeaderBuilder};

use std::io;
use std::result::Result::Ok;

const BZIP2_HEADER: [u8; 4] = [b'B', b'Z', b'h', b'1'];
const GZIP_HEADER: u16 = 0x1F8B;
const GZIP_CHUNK_READ_BUFFER_SIZE: usize = 512;

#[derive(Debug, Clone)]
pub enum CompressionType {
    None,
    Bzip2,
    Gzip,
}

impl CompressionType {
    pub fn get(code: u8) -> CompressionType {
        match code {
            1 => CompressionType::Bzip2,
            2 => CompressionType::Gzip,
            _ => CompressionType::None,
        }
    }
}

pub fn decompress(data: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    if data.is_empty() {
        panic!("cannot decompress empty data")
    }

    if data.len() < 5 {
        panic!("cannot decompress data with a length less than 5")
    }

    let mut buffer = ByteBuffer::from_bytes(&data[..]);

    // TODO checks
    let compression_type: CompressionType = CompressionType::get(buffer.read_u8()?);
    let compressed_size: usize = buffer.read_u32()? as usize;
    let decompressed_size: usize = buffer.read_u32()? as usize;

    match compression_type {
        CompressionType::Bzip2 => {
            decompress_bzip2(buffer.read_bytes(compressed_size)?, decompressed_size)
        }

        CompressionType::Gzip => unimplemented!("gzip decompression is not implemented"),
        CompressionType::None => unimplemented!("decompression of unknown type is not implemented"),
    }
}

pub fn decompress_bzip2(
    compressed_data: Vec<u8>,
    decompressed_size: usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    if compressed_data.is_empty() {
        panic!("cannot decompress empty data")
    }
    if compressed_data.len() < 5 {
        panic!("cannot decompress data with a length less than 5")
    }
    let compressed_data: Vec<u8> = [&BZIP2_HEADER, compressed_data.as_slice()].concat();
    let mut decoder = BzDecoder::new(&compressed_data[..]);
    let mut decompressed_data = vec![0; decompressed_size];
    decoder.read_exact(&mut decompressed_data)?;
    Ok(decompressed_data)
}

pub fn decompress_gzip(compressed_data: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
    if compressed_data.is_empty() {
        panic!("cannot decompress empty data")
    }

    let header = (compressed_data[0] as u16) << 8 | (compressed_data[1] as u16);
    if header != GZIP_HEADER {
        panic!("invalid gzip header")
    }

    let mut decoder = Decoder::new(&compressed_data[..])?;
    let mut decompressed_buffer = Vec::new();

    loop {
        let mut read_chunks: Vec<u8> = Vec::with_capacity(GZIP_CHUNK_READ_BUFFER_SIZE);
        let read = decoder
            .by_ref()
            .take(GZIP_CHUNK_READ_BUFFER_SIZE as u64)
            .read_to_end(&mut read_chunks)?;
        if read == 0 {
            break;
        }
        decompressed_buffer.append(&mut read_chunks);
        if read < GZIP_CHUNK_READ_BUFFER_SIZE {
            break;
        }
    }

    Ok(decompressed_buffer)
}

fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut encoder = Encoder::new(Vec::new()).unwrap();
    encoder.write_all(data);
    encoder.finish().into_result()
}

#[cfg(test)]
mod tests {
    use crate::compression::{compress_gzip, decompress, decompress_gzip};

    // TODO improve the tests

    #[test]
    fn test_bzip2_decompress() {
        let file_data = std::fs::read("./data/tests/hello_world_bzip.dat").unwrap();
        let decompressed_data = decompress(file_data).unwrap();
        let decompressed_to_string = String::from_utf8(decompressed_data).unwrap();
        assert_eq!(decompressed_to_string, "Hello world!");
    }

    #[test]
    fn test_gzip_compress_and_decompress() {
        let data = b"Hello world!";
        let compressed_data = compress_gzip(data).unwrap();
        let decompressed_data = decompress_gzip(compressed_data).unwrap();
        assert_eq!(
            String::from_utf8(decompressed_data).unwrap(),
            "Hello world!"
        );
    }
}
