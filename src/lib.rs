mod bytebuffer;
mod compression;
mod filesystem;
mod str;

// TODO proper tests
#[cfg(test)]
mod tests {
    use crate::filesystem::{Archive, FileSystem, ARCHIVE_INDEX_TYPE};

    #[test]
    fn wat() {
        let fs = FileSystem::new("./cache/").unwrap();
        let read_data = fs.read(ARCHIVE_INDEX_TYPE, 1).unwrap(); // title entry
        let archive = Archive::decode(read_data).unwrap();
    }
}
