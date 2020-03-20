pub mod archive;
pub mod compression;
pub mod error;
pub mod filesystem;
pub mod index;
mod str;
mod versionlist;

// TODO proper tests
#[cfg(test)]
mod tests {

    use crate::compression::decompress_gzip;
    use crate::filesystem::FileSystem;
    use crate::index::IndexType;
    use crate::versionlist::VersionList;

    use std::fs::File;
    use std::io::{Cursor, Write};

    #[test]
    fn archive_decoding() {
        let fs = FileSystem::new("./data/cache/").unwrap();
        let archive = fs.read_archive(5).unwrap();
        let anim_crc = archive.entry_name("anim_crc").unwrap();
        let _data = anim_crc.uncompressed_data();
    }

    #[test]
    fn error_testing() {
        let fs = FileSystem::new("./data/cache/").unwrap();
        let version_id = 5;

        let anim_version = fs.read_archive(version_id).unwrap();
        let _anim_version = anim_version.entry_name("anim_version").unwrap();
        let anim_version = VersionList::decode(&mut Cursor::new([9]), 5).unwrap();

        println!("{:?}", anim_version);
    }

    #[test]
    fn gzip_decoding() {
        let fs = FileSystem::new("./data/cache/").unwrap();
        let file_entry_id = 17;
        let read_data = fs.read(IndexType::MIDI, file_entry_id).unwrap();
        let decompressed_data = decompress_gzip(read_data).unwrap();
        let mut midi = File::create("./dump/midi/17.mid").unwrap();
        midi.write_all(&decompressed_data).unwrap();
    }

    #[test]
    fn gzip_large_decoding() {
        let fs = FileSystem::new("./data/cache_large/").unwrap();
        for model_id in 0..70_000 {
            let read_data = fs.read(IndexType::MODEL, model_id).unwrap();
            if !read_data.is_empty() {
                decompress_gzip(read_data).unwrap();
            }
        }
    }

    #[test]
    fn load_big() {
        let fs = FileSystem::new("./data/cache_large/").unwrap();
        for model_id in 0..70_000 {
            fs.read(IndexType::MODEL, model_id).unwrap();
        }
    }
}
