use crate::bytebuffer::ByteBufferExt;
use bytebuffer::ByteBuffer;
use std::collections::HashMap;
use std::error::Error;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const DEFAULT_DATA_FILE_NAME: &str = "main_file_cache.dat";
pub const DEFAULT_INDEX_FILE_PREFIX: &str = "main_file_cache.idx";
pub const MAX_INDEX_COUNT: u8 = 255;
pub const INDEX_FILE_BLOCK_SIZE: u8 = 6;
pub const TOTAL_BLOCK_LENGTH: u64 = 520;
pub const ARCHIVE_INDEX_TYPE: IndexType = IndexType(0);

pub struct IndexType(u8);

#[derive(Debug)]
pub struct FileSystem {
    main_data_file: File,
    indices: HashMap<u8, File>,
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

        let seek_from = SeekFrom::Start((entry_id as u64) * (6 as u64));

        let mut buffer: [u8; 6 as usize] = [0; 6 as usize];
        index_file.seek(seek_from)?;
        index_file.read(&mut buffer)?;

        let mut buffer = ByteBuffer::from_bytes(&buffer);

        let size: u32 = buffer.read_tri_byte()?;
        let offset: u64 = buffer.read_tri_byte()? as u64;

        println!("size: {}, offset: {}", size, offset);

        Ok(Index { size, offset })
    }
}
