pub mod archive;
mod bytebuffer;
pub mod compression;
pub mod filesystem;
pub mod index;
mod str;

// TODO proper tests
#[cfg(test)]
mod tests {

    use crate::compression::decompress_gzip;
    use crate::filesystem::FileSystem;
    use crate::index::IndexType;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn archive_decoding() {
        let fs = FileSystem::new("./data/cache/").unwrap();
        let archive = fs.read_archive(5).unwrap();
        let anim_crc = archive.entry_name("anim_crc").unwrap();
    }

    #[test]
    fn gzip_decoding() {
        let fs = FileSystem::new("./data/cache/").unwrap();
        let read_data = fs.read(IndexType::MIDI_INDEX_TYPE, 17).unwrap();
        let decompressed_data = decompress_gzip(read_data).unwrap();
        let mut midi = File::create("./dump/midi/17.mid").unwrap();
        midi.write_all(&decompressed_data).unwrap();
    }

    #[test]
    fn gzip_large_decoding() {
        let fs = FileSystem::new("./data/cache_large/").unwrap();
        for model_id in 0..70_000 {
            let read_data = fs.read(IndexType::MODEL_INDEX_TYPE, model_id).unwrap();
            if !read_data.is_empty() {
                decompress_gzip(read_data).unwrap();
            }
        }
    }

    #[test]
    fn load_big() {
        let fs = FileSystem::new("./data/cache_large/").unwrap();
        for model_id in 0..70_000 {
            fs.read(IndexType::MODEL_INDEX_TYPE, model_id).unwrap();
        }
    }
}
