use crate::archive::Archive;
use crate::error::FileSystemError;
use crate::index::{Index, IndexType};

use std::collections::HashMap;
use std::convert::TryFrom;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

// TODO should group these constants somehow
pub const DEFAULT_DATA_FILE_NAME: &str = "main_file_cache.dat";
pub const DEFAULT_INDEX_FILE_PREFIX: &str = "main_file_cache.idx";
pub const MAX_INDEX_COUNT: u8 = 255;
pub const TOTAL_BLOCK_SIZE: u64 = 520;
pub const BLOCK_CHUNK_SIZE: u32 = 512;
pub const BLOCK_CHUNK_EXTENDED_SIZE: u32 = 510;
pub const BLOCK_HEADER_SIZE: usize = 8;
pub const BLOCK_HEADER_EXTENDED_SIZE: usize = 10;

#[derive(Debug)]
pub struct FileSystem {
    main_data_file: File,
    indices: HashMap<u8, Index>,
}

struct CacheSectorHeader {
    next_entry_id: u32,
    next_sequence: u32,
    next_block: u64,
    next_index_id: u8,
}

impl TryFrom<&[u8]> for CacheSectorHeader {
    type Error = FileSystemError;

    fn try_from(block_data: &[u8]) -> Result<Self, Self::Error> {
        let (next_entry_id, next_sequence, next_block, next_index_id) = match block_data.len() {
            BLOCK_HEADER_EXTENDED_SIZE => (
                ((block_data[0] as u32) << 24)
                    | ((block_data[1] as u32) << 16)
                    | ((block_data[2] as u32) << 8)
                    | (block_data[3] as u32),
                (((block_data[4] as u32) << 8) | (block_data[5] as u32)),
                ((block_data[6] as u64) << 16)
                    | ((block_data[7] as u64) << 8)
                    | (block_data[8] as u64),
                block_data[9] as u8,
            ),
            BLOCK_HEADER_SIZE => (
                ((block_data[0] as u32) << 8) | (block_data[1] as u32),
                (((block_data[2] as u32) << 8) | (block_data[3] as u32)),
                ((block_data[4] as u64) << 16)
                    | ((block_data[5] as u64) << 8)
                    | (block_data[6] as u64),
                block_data[7] as u8,
            ),
            _ => {
                return Err(FileSystemError::msg(format!(
                    "Block header has invalid length of {}.",
                    block_data.len()
                )))
            }
        };
        Ok(CacheSectorHeader {
            next_entry_id,
            next_sequence,
            next_block,
            next_index_id,
        })
    }
}

impl FileSystem {
    pub fn new<P: AsRef<Path>>(base: P) -> Result<Self, FileSystemError> {
        let path = base.as_ref();
        let main_data_file_path = &path.join(DEFAULT_DATA_FILE_NAME);
        let main_data_file = File::open(main_data_file_path).map_err(|e| {
            FileSystemError::main_cache_file_not_found(e, PathBuf::from(&main_data_file_path))
        })?;
        let mut indices = HashMap::new();
        let index_file_path = |index_id: &u8| -> PathBuf {
            path.join(format!("{}{}", DEFAULT_INDEX_FILE_PREFIX, index_id))
        };
        indices.extend(
            (0..=MAX_INDEX_COUNT)
                .filter(|index_id| index_file_path(index_id).exists())
                .map(|index_id: u8| {
                    (
                        index_id,
                        Index::new(index_id, File::open(index_file_path(&index_id)).unwrap()),
                    )
                }),
        );
        Ok(FileSystem {
            main_data_file,
            indices,
        })
    }

    pub fn index(&self, index_type: IndexType) -> Result<&Index, FileSystemError> {
        let index_id = index_type.id();
        match self.indices.get(&index_id) {
            Some(index) => Ok(index),
            None => Err(FileSystemError::index_not_found(index_type)),
        }
    }

    pub fn file_count(&self, index_type: IndexType) -> Result<u64, FileSystemError> {
        let index = self.index(index_type)?;
        Ok(index.file_count())
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn read_archive(&self, entry_id: u32) -> Result<Archive, Box<dyn Error>> {
        let file_data = self.read(IndexType::ARCHIVE_INDEX_TYPE, entry_id)?;
        Archive::try_from(file_data)
    }

    pub fn read(&self, index_type: IndexType, entry_id: u32) -> Result<Vec<u8>, FileSystemError> {
        let index = self.index(index_type)?;
        let index_entry = index.entry(entry_id)?;
        let index_id = index.index_type().id();
        let ref mut main_data_file = &self.main_data_file;
        let mut buffer: Vec<u8> = Vec::with_capacity(index_entry.size() as usize);
        let mut block = index_entry.offset();
        let mut remaining_bytes = index_entry.size();
        let mut current_sequence = 0;
        // if the entry id is larger than a unsigned short integer (65535)
        let large = entry_id > u16::max_value() as u32;
        let block_header_size = if large {
            BLOCK_HEADER_EXTENDED_SIZE
        } else {
            BLOCK_HEADER_SIZE
        };
        let block_chunk_size = if large {
            BLOCK_CHUNK_EXTENDED_SIZE
        } else {
            BLOCK_CHUNK_SIZE
        };
        while remaining_bytes > 0 {
            let mut block_data: [u8; TOTAL_BLOCK_SIZE as usize] = [0; TOTAL_BLOCK_SIZE as usize];
            main_data_file.seek(SeekFrom::Start(block * TOTAL_BLOCK_SIZE))?;
            main_data_file.read(&mut block_data)?;
            let sector_header = CacheSectorHeader::try_from(&block_data[0..block_header_size])?;
            let chunks_consumed = std::cmp::min(remaining_bytes, block_chunk_size);
            if remaining_bytes > 0 {
                // TODO proper error checking
                if sector_header.next_index_id != (index_id + 1) {
                    return Err(FileSystemError::msg(format!(
                        "Index id mismatch. Expected: {}, actual: {}.",
                        (index_id + 1),
                        sector_header.next_index_id
                    )));
                }
                if sector_header.next_sequence != current_sequence {
                    return Err(FileSystemError::msg(format!(
                        "Sequence block mismatch. Expected {}, actual: {}.",
                        current_sequence, sector_header.next_sequence,
                    )));
                }
                if sector_header.next_entry_id != entry_id {
                    return Err(FileSystemError::msg(format!(
                        "File entry id mismatch. Expected {}, actual: {}.",
                        entry_id, sector_header.next_entry_id,
                    )));
                }
                buffer.write(&block_data[block_header_size..])?;
                remaining_bytes -= chunks_consumed;
                block = sector_header.next_block;
                current_sequence += 1;
            }
        }
        Ok(buffer)
    }
}
