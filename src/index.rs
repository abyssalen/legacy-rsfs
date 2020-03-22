use crate::errors::FileSystemError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct IndexType(u8);

impl IndexType {
    pub const ARCHIVE: IndexType = IndexType(0);
    pub const MODEL: IndexType = IndexType(1);
    pub const ANIMATION: IndexType = IndexType(2);
    pub const MIDI: IndexType = IndexType(3);
    pub const MAP: IndexType = IndexType(4);

    pub fn new(index_id: u8) -> Self {
        IndexType(index_id)
    }

    pub fn id(&self) -> u8 {
        self.0
    }
}

#[derive(Debug)]
pub struct Index {
    index_type: IndexType,
    file: File,
    file_size: u64,
}

impl Index {
    pub const SIZE: u8 = 6;

    pub(crate) fn new(index_id: u8, file: File) -> Self {
        let file_size = file.metadata().unwrap().len();
        Index {
            index_type: IndexType(index_id),
            file,
            file_size,
        }
    }

    pub fn entry(&self, entry_id: u32) -> Result<IndexEntry, FileSystemError> {
        let ptr = (entry_id as u64) * (Index::SIZE as u64);
        if ptr >= self.file_size {
            return Err(FileSystemError::IndexEntryNotFound(entry_id));
        }
        let mut index_file = &self.file;
        let seek_from = SeekFrom::Start(ptr);
        let mut buffer: [u8; Index::SIZE as usize] = [0; Index::SIZE as usize];
        index_file.seek(seek_from)?;
        index_file.read(&mut buffer)?;
        let size: u32 = ((buffer[0] as u32) << 16) | ((buffer[1] as u32) << 8) | (buffer[2] as u32);
        let offset: u64 =
            ((buffer[3] as u64) << 16) | ((buffer[4] as u64) << 8) | (buffer[5] as u64);
        Ok(IndexEntry {
            id: entry_id,
            size,
            offset,
        })
    }

    pub fn index_type(&self) -> &IndexType {
        &self.index_type
    }

    pub fn file_count(&self) -> u64 {
        self.file_size / Index::SIZE as u64
    }
}

pub struct IndexEntry {
    id: u32,
    size: u32,
    offset: u64,
}

impl IndexEntry {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }
}
