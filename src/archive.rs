use crate::compression;
use crate::str::StrExt;

use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::io::{Cursor, Read};

pub const ARCHIVE_HEADER_SIZE: usize = 6;

#[derive(Debug)]
pub struct Archive {
    entries: HashMap<i32, ArchiveEntry>,
    header: ArchiveHeader,
}

impl Archive {
    pub fn entry_name(&self, name: &str) -> Option<&ArchiveEntry> {
        self.entries.get(&name.name_hash())
    }

    pub fn entry_hash(&self, identifier: i32) -> Option<&ArchiveEntry> {
        self.entries.get(&identifier)
    }

    pub fn entry_count(&self) -> usize {
        self.entries.iter().count()
    }
}

#[derive(Debug)]
struct ArchiveHeader {
    decompressed_size: u32,
    compressed_size: u32,
}

impl TryFrom<&[u8; ARCHIVE_HEADER_SIZE]> for ArchiveHeader {
    type Error = Box<dyn Error>;

    fn try_from(value: &[u8; ARCHIVE_HEADER_SIZE]) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(value);
        let decompressed_size = cursor.read_u24::<BigEndian>()?;
        let compressed_size = cursor.read_u24::<BigEndian>()?;
        Ok(ArchiveHeader {
            decompressed_size,
            compressed_size,
        })
    }
}

#[derive(Debug)]
pub struct ArchiveEntry {
    identifier: i32,
    uncompressed_size: u32,
    compressed_size: u32,
    uncompressed_data: Vec<u8>,
}

impl ArchiveEntry {
    pub fn uncompressed_data(&self) -> &[u8] {
        &self.uncompressed_data
    }

    pub fn identifier(&self) -> i32 {
        self.identifier
    }
}

impl TryFrom<Vec<u8>> for Archive {
    type Error = Box<dyn Error>;

    fn try_from(buffer: Vec<u8>) -> Result<Self, Self::Error> {
        // TODO proper handling of errors
        if buffer.is_empty() {
            panic!("given archive is empty!");
        }

        let mut buffer = Cursor::new(buffer);

        let mut header: [u8; ARCHIVE_HEADER_SIZE] = [0; ARCHIVE_HEADER_SIZE];
        buffer.read_exact(&mut header)?;
        let header = ArchiveHeader::try_from(&header).unwrap();
        let (decompressed_size, compressed_size) = (
            header.decompressed_size as usize,
            header.compressed_size as usize,
        );

        let mut extracted = false;
        if decompressed_size != compressed_size {
            let mut compressed_data = vec![0; compressed_size];
            buffer.read_exact(&mut compressed_data)?;
            let decompressed_data =
                compression::decompress_bzip2(compressed_data, decompressed_size)?;
            buffer = Cursor::new(decompressed_data);
            extracted = true;
        }

        let entries_count = buffer.read_u16::<BigEndian>()? as usize;

        let mut entries: HashMap<i32, ArchiveEntry> = HashMap::with_capacity(entries_count);
        let mut identifiers: Vec<i32> = vec![0; entries_count];
        let mut uncompressed_sizes = vec![0; entries_count];
        let mut compressed_sizes = vec![0; entries_count];

        for entry_index_id in 0..entries_count {
            identifiers[entry_index_id] = buffer.read_i32::<BigEndian>()?;
            uncompressed_sizes[entry_index_id] = buffer.read_u24::<BigEndian>()?;
            compressed_sizes[entry_index_id] = buffer.read_u24::<BigEndian>()?;
        }

        // decompress the archive entries
        for entry_index_id in 0..entries_count {
            let (identifier, uncompressed_size, compressed_size) = (
                identifiers[entry_index_id],
                uncompressed_sizes[entry_index_id] as usize,
                compressed_sizes[entry_index_id] as usize,
            );

            let data = if extracted {
                let mut data = vec![0; uncompressed_size];
                buffer.read_exact(&mut data)?;
                data
            } else {
                let mut data = vec![0; compressed_size];
                buffer.read_exact(&mut data)?;
                compression::decompress_bzip2(data, uncompressed_size)?
            };
            entries.insert(
                identifier,
                ArchiveEntry {
                    identifier,
                    uncompressed_size: uncompressed_size as u32,
                    compressed_size: compressed_size as u32,
                    uncompressed_data: data,
                },
            );
        }
        Ok(Archive { entries, header })
    }
}
