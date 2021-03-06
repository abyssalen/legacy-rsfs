pub mod archive;
pub mod compression;
mod errors;
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

    use crate::archive::ArchiveType;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn archive_decoding() {
        let fs = FileSystem::new("./data/cache/").unwrap();
        let title_archive = fs.read_archive(ArchiveType::TITLE).unwrap();
        let logo_archive_entry = title_archive.entry_name("logo.dat").unwrap();
        println!("{:#?}", logo_archive_entry);
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
