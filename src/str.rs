pub trait StrExt {
    fn name_hash(&self) -> i32;
}

impl StrExt for str {
    fn name_hash(&self) -> i32 {
        self.to_uppercase()
            .chars()
            .fold(0, |hash, char| (hash.wrapping_mul(61)) + (char as i32) - 32)
    }
}

#[cfg(test)]
mod tests {
    use crate::str::StrExt;

    #[test]
    fn test_archive_entry_name_positive_hash() {
        assert_eq!("mapedge.dat".name_hash(), 1362520410)
    }
    #[test]
    fn test_archive_entry_name_negative_hash() {
        assert_eq!("invback.dat".name_hash(), -1568083395)
    }
}
