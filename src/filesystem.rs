use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub const DEFAULT_DATA_FILE_NAME: &str = "main_file_cache.dat";
pub const DEFAULT_INDEX_FILE_PREFIX: &str = "main_file_cache.idx";
pub const MAX_INDEX_COUNT: u8 = 255;

pub const INDEX_FILE_BLOCK_SIZE: u8 = 6;

pub const TOTAL_BLOCK_LENGTH: u64 = 520;

#[derive(Debug)]
pub struct FileSystem {
    main_data_file: File,
    indices: HashMap<u8, File>,
}

pub struct Index {
    size: u32,
    offset: usize,
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
}
