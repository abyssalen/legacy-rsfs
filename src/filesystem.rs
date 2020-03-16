use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use bytebuffer::ByteBuffer;

use crate::bytebuffer::ByteBufferExt;

// TODO should group these constants somehow
pub const DEFAULT_DATA_FILE_NAME: &str = "main_file_cache.dat";
pub const DEFAULT_INDEX_FILE_PREFIX: &str = "main_file_cache.idx";
pub const MAX_INDEX_COUNT: u8 = 255;
pub const INDEX_FILE_BLOCK_SIZE: u8 = 6;
pub const TOTAL_BLOCK_SIZE: u64 = 520;
pub const ARCHIVE_INDEX_TYPE: IndexType = IndexType(0);
pub const BLOCK_CHUNK_SIZE: u32 = 512;
pub const BLOCK_CHUNK_LARGE_SIZE: u32 = 510;
pub const BLOCK_HEADER_SIZE: u8 = 8;
pub const BLOCK_HEADER_LARGE_SIZE: u8 = 10;

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
    data: Vec<u8>,
}

impl Archive {
    pub fn decode(buffer: Vec<u8>) -> Result<Archive, Box<dyn Error>> {
        // TODO proper handling of errors
        if buffer.is_empty() {
            panic!("given archive is empty!");
        }

        let mut buffer = ByteBuffer::from_bytes(&buffer[..]);
        let mut extracted = false;

        let raw_size = buffer.read_tri_byte()?;
        let real_size = buffer.read_tri_byte()?;

        if raw_size != real_size {
            // TODO extracting
            extracted = true;
        }

        let entries_count = buffer.read_u16()? as usize;

        let mut entries: HashMap<i32, ArchiveEntry> = HashMap::with_capacity(entries_count);

        let mut identifiers: Vec<i32> = vec![0; entries_count];
        let mut raw_sizes = vec![0; entries_count];
        let mut real_sizes = vec![0; entries_count];

        for entry_index_id in 0..entries_count {
            identifiers[entry_index_id] = buffer.read_i32()?;
            raw_sizes[entry_index_id] = buffer.read_tri_byte()?;
            real_sizes[entry_index_id] = buffer.read_tri_byte()?;
        }

        for entry_index_id in 0..entries_count {
            let (identifier, raw_size, real_size) = (
                identifiers[entry_index_id],
                raw_sizes[entry_index_id],
                real_sizes[entry_index_id],
            );
            let actual_entry_size = if extracted { raw_size } else { real_size };
            let data = buffer.read_bytes(actual_entry_size as usize)?;
            let uncompressed_data = data;
            // TODO decompress the archive entries
            entries.insert(
                identifier,
                ArchiveEntry {
                    identifier,
                    data: uncompressed_data,
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

    pub fn get_index(
        &self,
        index_type: &IndexType,
        entry_id: u32,
    ) -> Result<Index, Box<dyn Error>> {
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

        let mut buffer = ByteBuffer::from_bytes(&buffer);
        let size: u32 = buffer.read_tri_byte()?;
        let offset: u64 = buffer.read_tri_byte()? as u64;

        Ok(Index { size, offset })
    }

    pub fn read(&self, index_type: IndexType, entry_id: u32) -> Result<Vec<u8>, Box<dyn Error>> {
        // TODO should check for errors!!!
        let IndexType(index_id) = index_type;
        let index = self.get_index(&index_type, entry_id).unwrap();
        let ref mut main_data_file = &self.main_data_file;

        let mut buffer: ByteBuffer = ByteBuffer::new();
        buffer.resize(index.size as usize);

        let mut block_data_buffer = ByteBuffer::new();
        block_data_buffer.resize(TOTAL_BLOCK_SIZE as usize);

        let mut block = index.offset;
        let mut remaining_bytes = index.size;
        let mut current_sequence = 0;

        let large = entry_id > 65535;

        while remaining_bytes > 0 {
            let mut block_data: [u8; TOTAL_BLOCK_SIZE as usize] = [0; TOTAL_BLOCK_SIZE as usize];
            main_data_file.seek(SeekFrom::Start(block * TOTAL_BLOCK_SIZE))?;
            main_data_file.read(&mut block_data)?;
            block_data_buffer.write(&block_data)?;

            let (next_entry_id, next_sequence, next_block, next_index_id) = (
                if large {
                    block_data_buffer.read_u32()?
                } else {
                    block_data_buffer.read_u16()? as u32
                },
                block_data_buffer.read_u16()?,
                block_data_buffer.read_tri_byte()? as u64,
                block_data_buffer.read_u8()?,
            );

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
                    block_data_buffer.read_bytes(remaining_chunk_size_left as usize)?[..].as_mut(),
                )?;

                // clear the block's data buffer by setting all of the underlying data to 0
                // we don't want to use ByteBuffer.clear() because it truncates the underlying Vec
                // we would have to resize the underlying Vec again which isn't efficient
                block_data_buffer.set_wpos(0); // ensure the writing cursor position is at the beginning of the buffer
                block_data_buffer.write_bytes(&[0; TOTAL_BLOCK_SIZE as usize]);
                // prepares the block data buffer for the next block data to be read by
                // resetting the buffer's reading and writing cursor positions
                block_data_buffer.set_wpos(0);
                block_data_buffer.set_rpos(0);

                remaining_bytes -= remaining_chunk_size_left;
                block = next_block;
                current_sequence += 1;
            }
        }
        block_data_buffer.clear();
        Ok(buffer.to_bytes())
    }
}
