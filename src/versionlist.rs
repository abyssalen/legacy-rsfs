use byteorder::{BigEndian, ReadBytesExt};

use crate::error::FileSystemError;

use std::io::Read;

pub const DEFAULT_VERSION_ENTRY_NAMES: &'static [&'static str; 4] = &[
    "model_version",
    "anim_version",
    "midi_version",
    "map_version",
];
pub const DEFAULT_CRC_ENTRY_NAMES: &'static [&'static str; 4] =
    &["model_crc", "anim_crc", "midi_crc", "map_crc"];

#[derive(Debug)]
pub struct VersionList {
    versions: Vec<u32>,
}

impl VersionList {
    pub fn decode<R: Read>(reader: &mut R, len: usize) -> Result<Self, FileSystemError> {
        let count = len / 2;
        let mut versions: Vec<u32> = vec![0; count];
        for i in 0..count {
            versions[i] = reader.read_u16::<BigEndian>()? as u32;
        }
        Ok(VersionList { versions })
    }

    pub fn get(&self, file_id: u32) -> Option<&u32> {
        self.versions.get(file_id as usize)
    }

    pub fn len(&self) -> usize {
        self.versions.len()
    }
}

#[derive(Debug)]
pub struct CrcList {
    crcs: Vec<u32>,
}

impl CrcList {
    pub fn decode<R: Read>(reader: &mut R, len: usize) -> Result<Self, FileSystemError> {
        let count = len / 4;
        let mut crcs: Vec<u32> = vec![0; count];
        for i in 0..count {
            crcs[i] = reader.read_u32::<BigEndian>()?;
        }
        Ok(CrcList { crcs })
    }

    pub fn get(&self, file_id: u32) -> Option<&u32> {
        self.crcs.get(file_id as usize)
    }

    pub fn len(&self) -> usize {
        self.crcs.len()
    }
}
