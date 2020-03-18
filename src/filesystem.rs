use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use bytebuffer::ByteBuffer;

use crate::bytebuffer::ByteBufferExt;
use crate::compression;
use crate::str::StrExt;

// TODO should group these constants somehow
pub const DEFAULT_DATA_FILE_NAME: &str = "main_file_cache.dat";
pub const DEFAULT_INDEX_FILE_PREFIX: &str = "main_file_cache.idx";
pub const MAX_INDEX_COUNT: u8 = 255;
pub const INDEX_FILE_BLOCK_SIZE: u8 = 6;
pub const TOTAL_BLOCK_SIZE: u64 = 520;
pub const BLOCK_CHUNK_SIZE: u32 = 512;
pub const BLOCK_CHUNK_LARGE_SIZE: u32 = 510;
pub const BLOCK_HEADER_SIZE: u8 = 8;
pub const BLOCK_HEADER_LARGE_SIZE: u8 = 10;

pub const ARCHIVE_INDEX_TYPE: IndexType = IndexType(0);
pub const MODEL_INDEX_TYPE: IndexType = IndexType(1);
pub const ANIMATION_INDEX_TYPE: IndexType = IndexType(2);
pub const MIDI_INDEX_TYPE: IndexType = IndexType(3);
pub const MAP_INDEX_TYPE: IndexType = IndexType(4);

pub struct IndexType(u8);

#[derive(Debug)]
pub struct FileSystem {
    main_data_file: File,
    indices: HashMap<u8, File>,
}

#[derive(Debug)]
pub struct Archive {
    entries: HashMap<i32, ArchiveEntry>,
}
#[derive(Debug)]
pub struct ArchiveEntry {
    identifier: i32,
    uncompressed_data: Vec<u8>,
}

impl Archive {
    pub fn decode(buffer: Vec<u8>) -> Result<Archive, Box<dyn Error>> {
        // TODO proper handling of errors
        if buffer.is_empty() {
            panic!("given archive is empty!");
        }

        let mut buffer = ByteBuffer::from_bytes(&buffer[..]);
        let mut extracted = false;

        let uncompressed_size: usize = buffer.read_tri_byte()? as usize;
        let real_size: usize = buffer.read_tri_byte()? as usize;

        if uncompressed_size != real_size {
            let compressed_data = buffer.read_bytes(real_size)?;
            buffer.clear();
            buffer.write_bytes(&compression::decompress_bzip2(
                compressed_data,
                uncompressed_size,
            )?);
            extracted = true;
        }

        let entries_count = buffer.read_u16()? as usize;

        let mut entries: HashMap<i32, ArchiveEntry> = HashMap::with_capacity(entries_count);
        let mut identifiers: Vec<i32> = vec![0; entries_count];
        let mut uncompressed_sizes = vec![0; entries_count];
        let mut compressed_sizes = vec![0; entries_count];

        for entry_index_id in 0..entries_count {
            identifiers[entry_index_id] = buffer.read_i32()?;
            uncompressed_sizes[entry_index_id] = buffer.read_tri_byte()?;
            compressed_sizes[entry_index_id] = buffer.read_tri_byte()?;
        }

        // decompress the archive entries
        for entry_index_id in 0..entries_count {
            let (identifier, uncompressed_size, compressed_size) = (
                identifiers[entry_index_id],
                uncompressed_sizes[entry_index_id] as usize,
                compressed_sizes[entry_index_id] as usize,
            );
            let data = if extracted {
                buffer.read_bytes(uncompressed_size)?
            } else {
                compression::decompress_bzip2(
                    buffer.read_bytes(compressed_size)?,
                    uncompressed_size,
                )?
            };
            entries.insert(
                identifier,
                ArchiveEntry {
                    identifier,
                    uncompressed_data: data,
                },
            );
        }
        Ok(Archive { entries })
    }
}

pub struct Index {
    size: u32,
    offset: u64,
}

impl Index {}

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
                .map(|index_id: u8| (index_id, File::open(index_file_path(&index_id)).unwrap())),
        );
        Ok(FileSystem {
            main_data_file,
            indices,
        })
    }

    pub fn get_index(&self, index_type: IndexType, entry_id: u32) -> Result<Index, Box<dyn Error>> {
        let IndexType(index_id) = index_type;
        let mut index_file = match self.indices.get(&index_id) {
            Some(index_file) => index_file,
            // TODO actual error handling
            None => panic!("bro! can't get the index file rip"),
        };
        let seek_from = SeekFrom::Start((entry_id as u64) * (INDEX_FILE_BLOCK_SIZE as u64));
        let mut buffer: [u8; INDEX_FILE_BLOCK_SIZE as usize] = [0; INDEX_FILE_BLOCK_SIZE as usize];
        index_file.seek(seek_from)?;
        index_file.read(&mut buffer)?;
        let size: u32 = ((buffer[0] as u32) << 16) | ((buffer[1] as u32) << 8) | (buffer[2] as u32);
        let offset: u64 =
            ((buffer[3] as u64) << 16) | ((buffer[4] as u64) << 8) | (buffer[5] as u64);
        Ok(Index { size, offset })
    }

    pub fn read(&self, index_type: IndexType, entry_id: u32) -> Result<Vec<u8>, Box<dyn Error>> {
        // TODO should check for errors!!!
        let IndexType(index_id) = index_type;
        let index = self.get_index(index_type, entry_id).unwrap();
        let ref mut main_data_file = &self.main_data_file;

        let mut buffer: Vec<u8> = Vec::with_capacity(index.size as usize);

        let mut block = index.offset;
        let mut remaining_bytes = index.size;
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
                if next_index_id != (index_id + 1)
                    || next_sequence != current_sequence
                    || next_entry_id != entry_id
                {
                    // TODO proper error checking
                    panic!(
                        "next_index_id: {}, should be: {}.... next_seq: {}, should be: {}.... \
                    next_entry: {}, should be: {}",
                        next_index_id,
                        index_id,
                        next_sequence,
                        current_sequence,
                        next_entry_id,
                        entry_id
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
