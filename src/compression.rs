use std::error::Error;
use std::io::Read;

use bytebuffer::ByteBuffer;
use bzip2::read::BzDecoder;

const BZIP2_HEADER: [u8; 4] = [b'B', b'Z', b'h', b'1'];

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
    let compressed_size = buffer.read_u32()?;
    let decompressed_size = buffer.read_u32()?;

    println!(
        "{:#?} {} {}",
        compression_type, compressed_size, decompressed_size
    );

    match compression_type {
        CompressionType::Bzip2 => {
            let compressed_data: Vec<u8> = [
                &BZIP2_HEADER,
                buffer.read_bytes(compressed_size as usize)?.as_slice(),
            ]
            .concat();

            let mut decompressor = BzDecoder::new(&compressed_data[..]);
            let mut decompressed_data = vec![0; decompressed_size as usize];
            decompressor.read_exact(&mut decompressed_data)?;
            Ok(decompressed_data)
        }

        // TODO fix these placeholder empty Vecs
        CompressionType::Gzip => Ok(vec![]),
        CompressionType::None => Ok(vec![]),
    }
}

#[cfg(test)]
mod tests {
    use crate::compression::decompress;

    // TODO improve the tests

    #[test]
    fn test_bzip2_decompress() {
        let file_data = std::fs::read("./data/tests/hello_world_bzip.dat").unwrap();
        let decompressed_data = decompress(file_data).unwrap();
        let decompressed_to_string = String::from_utf8(decompressed_data).unwrap();
        assert_eq!(decompressed_to_string, "Hello world!");
    }
}
