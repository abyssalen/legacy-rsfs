use std::error::Error;
use std::io::{Read, Write};

use bzip2::read::{BzDecoder, BzEncoder};
use flate2::write::GzEncoder;

use bzip2::Compression;
use flate2::read::GzDecoder;
use std::io;
use std::result::Result::Ok;

const BZIP2_HEADER_SIZE: usize = 4;
const BZIP2_HEADER: [u8; BZIP2_HEADER_SIZE] = [b'B', b'Z', b'h', b'1'];
const GZIP_HEADER: u16 = 0x1F8B;

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
    let mut decoder = GzDecoder::new(&compressed_data[..]);
    let mut result = Vec::with_capacity(compressed_data.len());
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

pub fn compress_bzip2(data: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut encoder = BzEncoder::new(data, Compression::Default);
    let mut result = Vec::with_capacity(data.len());
    encoder.read_to_end(&mut result)?;
    result.drain(0..BZIP2_HEADER_SIZE);
    Ok(result)
}

pub fn compress_gzip(data: &[u8]) -> Result<Vec<u8>, io::Error> {
    let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

#[cfg(test)]
mod tests {
    use crate::compression::{compress_bzip2, compress_gzip, decompress_bzip2, decompress_gzip};

    #[test]
    fn test_gzip_compression() {
        let data = b"Hello world!";
        let compressed_data = compress_gzip(data).unwrap();
        let decompressed_data = decompress_gzip(compressed_data).unwrap();
        assert_eq!(
            String::from_utf8(decompressed_data).unwrap(),
            "Hello world!"
        );
    }

    #[test]
    fn test_bzip2_compression() {
        let data = b"Hello world!";
        let decompressed_size = data.len();
        let compressed_data = compress_bzip2(data).unwrap();
        let decompressed_data = decompress_bzip2(compressed_data, decompressed_size).unwrap();
        assert_eq!(
            String::from_utf8(decompressed_data).unwrap(),
            "Hello world!"
        );
    }
}
