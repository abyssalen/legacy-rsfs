use crate::filesystem::{BLOCK_HEADER_EXTENDED_SIZE, BLOCK_HEADER_SIZE};
use crate::index::IndexType;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("File system IO compression error.")]
    CompressionError(#[from] CompressionError),
    #[error("File system IO error.")]
    Io(#[from] io::Error),
    #[error("Cannot create an Archive from empty data.")]
    EmptyArchiveDataGiven,
    #[error("{0} not found.")]
    DataFileNotFound(String),
    #[error("Could not find index {} in cache.", index_type.id())]
    IndexNotFound { index_type: IndexType },
    #[error("Could not find index entry {0} in cache.")]
    IndexEntryNotFound(u32),
    #[error("Could not find archive {0} in cache.")]
    ArchiveNotFound(u32),
    #[error(
        "Invalid block header length of {0}. It should be {} or {}",
        BLOCK_HEADER_SIZE,
        BLOCK_HEADER_EXTENDED_SIZE
    )]
    InvalidBlockHeaderLength(usize),
    #[error(
        "Sector {} mismatch. Expected: {}, actual: {}.",
        data_type,
        actual,
        expected
    )]
    SectorReadingDataMismatch {
        data_type: String,
        expected: usize,
        actual: usize,
    },
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Compression IO error.")]
    Io(#[from] io::Error),
    #[error("Cannot decompress empty data.")]
    EmptyCompressedDataGiven,
    #[error(
        "Invalid data length of {} given. Expected at least {}",
        given,
        expected_at_least
    )]
    InvalidCompressedDataLengthGiven {
        given: usize,
        expected_at_least: usize,
    },
    #[error("Invalid GZIP header from the given data.")]
    InvalidGZIPHeader,
}
