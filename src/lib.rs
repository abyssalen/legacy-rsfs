mod bytebuffer;
mod filesystem;

// TODO proper tests
#[cfg(test)]
mod tests {
    use crate::filesystem::{FileSystem, Index};

    #[test]
    fn wat() {
        let fs = FileSystem::new("./cache/").unwrap();

        //   fs.read(ARCHIVE_INDEX, 1);

        /*

        let fs = file_system.new("./cache/");

        fs.get_archive().get_group()

        */
    }
}
