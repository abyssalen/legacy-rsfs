use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::archive::Archive;

use crate::index::{Index, IndexType};

use std::convert::TryFrom;

// TODO should group these constants somehow
pub const DEFAULT_DATA_FILE_NAME: &str = "main_file_cache.dat";
pub const DEFAULT_INDEX_FILE_PREFIX: &str = "main_file_cache.idx";
pub const MAX_INDEX_COUNT: u8 = 255;
pub const TOTAL_BLOCK_SIZE: u64 = 520;
pub const BLOCK_CHUNK_SIZE: u32 = 512;
pub const BLOCK_CHUNK_LARGE_SIZE: u32 = 510;
pub const BLOCK_HEADER_SIZE: u8 = 8;
pub const BLOCK_HEADER_LARGE_SIZE: u8 = 10;

#[derive(Debug)]
pub struct FileSystem {
    main_data_file: File,
    indices: HashMap<u8, Index>,
}

impl FileSystem {
    pub fn new<P: AsRef<Path>>(base: P) -> Result<Self, Box<dyn Error>> {
        let path = base.as_ref();
        // TODO proper error checking
        let main_data_file = File::open(path.join(DEFAULT_DATA_FILE_NAME))?;
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

    pub fn index(&self, index_type: IndexType) -> Result<&Index, Box<dyn Error>> {
        let index_id = index_type.id();
        match self.indices.get(&index_id) {
            Some(index) => Ok(index),
            None => panic!("can't get the index {:#?}", index_type),
        }
    }

    pub fn read_archive(&self, entry_id: u32) -> Result<Archive, Box<dyn Error>> {
        let file_data = self.read(IndexType::ARCHIVE_INDEX_TYPE, entry_id)?;
        Archive::try_from(file_data)
    }

    pub fn read(&self, index_type: IndexType, entry_id: u32) -> Result<Vec<u8>, Box<dyn Error>> {
        // TODO should check for errors!!!
        let index = self.index(index_type)?;
        let index_entry = index.entry(entry_id)?;
        let index_id = index.index_type().id();
        let ref mut main_data_file = &self.main_data_file;

        let mut buffer: Vec<u8> = Vec::with_capacity(index_entry.size() as usize);

        let mut block = index_entry.offset();
        let mut remaining_bytes = index_entry.size();
        let mut current_sequence = 0;
        let large = entry_id > 65535;

        while remaining_bytes > 0 {
            let mut block_data: [u8; TOTAL_BLOCK_SIZE as usize] = [0; TOTAL_BLOCK_SIZE as usize];
            main_data_file.seek(SeekFrom::Start(block * TOTAL_BLOCK_SIZE))?;
            main_data_file.read(&mut block_data)?;

            let (next_entry_id, next_sequence, next_block, next_index_id) = if large {
                (
                    ((block_data[0] as u32) << 24)
                        | ((block_data[1] as u32) << 16)
                        | ((block_data[2] as u32) << 8)
                        | (block_data[3] as u32),
                    (((block_data[4] as u32) << 8) | (block_data[5] as u32)),
                    ((block_data[6] as u64) << 16)
                        | ((block_data[7] as u64) << 8)
                        | (block_data[8] as u64),
                    block_data[9] as u8,
                )
            } else {
                (
                    ((block_data[0] as u32) << 8) | (block_data[1] as u32),
                    (((block_data[2] as u32) << 8) | (block_data[3] as u32)),
                    ((block_data[4] as u64) << 16)
                        | ((block_data[5] as u64) << 8)
                        | (block_data[6] as u64),
                    block_data[7] as u8,
                )
            };

            let remaining_chunk_size_left = std::cmp::min(
                remaining_bytes,
                if large {
                    BLOCK_CHUNK_LARGE_SIZE
                } else {
                    BLOCK_CHUNK_SIZE
                },
            );

            if remaining_bytes > 0 {
                // TODO proper error checking

                if next_index_id != (index_id + 1) {
                    panic!(
                        "next index id: {} does not equal index id {}",
                        next_index_id,
                        (index_id + 1)
                    )
                }

                if next_sequence != current_sequence {
                    panic!(
                        "next seq: {} does not equal cur seq {}",
                        next_sequence, current_sequence
                    )
                }

                if next_entry_id != entry_id {
                    panic!(
                        "next entry id: {} does not equal cur entry id {}",
                        next_entry_id, entry_id
                    )
                }

                buffer.write(
                    &block_data[(if large {
                        BLOCK_HEADER_LARGE_SIZE
                    } else {
                        BLOCK_HEADER_SIZE
                    } as usize)..],
                )?;

                remaining_bytes -= remaining_chunk_size_left;
                block = next_block;
                current_sequence += 1;
            }
        }
        Ok(buffer)
    }
}
