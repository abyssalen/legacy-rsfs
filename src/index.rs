use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct IndexType(u8);

impl IndexType {
    pub const ARCHIVE_INDEX_TYPE: IndexType = IndexType(0);
    pub const MODEL_INDEX_TYPE: IndexType = IndexType(1);
    pub const ANIMATION_INDEX_TYPE: IndexType = IndexType(2);
    pub const MIDI_INDEX_TYPE: IndexType = IndexType(3);
    pub const MAP_INDEX_TYPE: IndexType = IndexType(4);

    pub fn id(&self) -> u8 {
        self.0
    }
}

#[derive(Debug)]
pub struct Index {
    index_type: IndexType,
    file: File,
}

impl Index {
    pub const INDEX_SIZE: u8 = 6;

    pub fn new(index_id: u8, file: File) -> Self {
        Index {
            index_type: IndexType(index_id),
            file,
        }
    }

    pub fn entry(&self, entry_id: u32) -> Result<IndexEntry, Box<dyn Error>> {
        let mut index_file = &self.file;
        let seek_from = SeekFrom::Start((entry_id as u64) * (Index::INDEX_SIZE as u64));
        let mut buffer: [u8; Index::INDEX_SIZE as usize] = [0; Index::INDEX_SIZE as usize];
        index_file.seek(seek_from)?;
        index_file.read(&mut buffer)?;
        let size: u32 = ((buffer[0] as u32) << 16) | ((buffer[1] as u32) << 8) | (buffer[2] as u32);
        let offset: u64 =
            ((buffer[3] as u64) << 16) | ((buffer[4] as u64) << 8) | (buffer[5] as u64);
        Ok(IndexEntry { size, offset })
    }

    pub fn index_type(&self) -> &IndexType {
        &self.index_type
    }

    pub fn file_count(&self) -> u64 {
        self.file.metadata().unwrap().len() / Index::INDEX_SIZE as u64
    }
}

pub struct IndexEntry {
    size: u32,
    offset: u64,
}

impl IndexEntry {
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }
}
