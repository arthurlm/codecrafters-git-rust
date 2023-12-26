use std::path::PathBuf;

pub fn checksum_to_path(cs: &str) -> PathBuf {
    assert_eq!(cs.len(), 40, "Invalid checksum size");
    PathBuf::from(&cs[..2]).join(&cs[2..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_to_path() {
        assert_eq!(
            checksum_to_path("e547aac8945402134e4c0b9bb85ad82361eed68a"),
            PathBuf::from("e5/47aac8945402134e4c0b9bb85ad82361eed68a"),
        );
    }
}
