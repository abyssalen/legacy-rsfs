mod bytebuffer;
mod filesystem;

// TODO proper tests
#[cfg(test)]
mod tests {
    use crate::filesystem::{FileSystem, ARCHIVE_INDEX_TYPE};

    #[test]
    fn wat() {
        let fs = FileSystem::new("./cache/").unwrap();

        fs.read(ARCHIVE_INDEX_TYPE, 1); // title entry

        /*

        let fs = file_system.new("./cache/");

        fs.get_archive().get_group()

        */
    }
}
