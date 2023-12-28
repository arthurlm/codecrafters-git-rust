use bytes::{Buf, Bytes};
use git_starter_rust::{
    pack_file::{read_object_size, PackFile},
    GitError,
};

fn read_packet(data: &'static [u8]) {
    PackFile::read(&mut Bytes::from_static(data).reader()).unwrap()
}

fn read_err_packet(data: &'static [u8]) -> GitError {
    PackFile::read(&mut Bytes::from_static(data).reader()).unwrap_err()
}

#[test]
fn test_invalid_parse() {
    assert_eq!(
        read_err_packet(b""),
        GitError::io("failed to fill whole buffer")
    );
    assert_eq!(
        read_err_packet(b"PUCK"),
        GitError::invalid_content("Invalid magic PACK")
    );
    assert_eq!(
        read_err_packet(b"PACK"),
        GitError::io("failed to fill whole buffer")
    );
    assert_eq!(
        read_err_packet(b"PACK\0\0\0\0"),
        GitError::invalid_content("Invalid PACK version")
    );
}

#[test]
fn test_valid_parse() {
    read_packet(include_bytes!("./data/sqlite-rust.pack"));
}

#[test]
fn test_read_object_size() {
    fn read(data: &'static [u8]) -> (u8, usize) {
        read_object_size(&mut Bytes::from_static(data).reader()).unwrap()
    }

    assert_eq!(read(&[0b0000_0000]), (0, 0));
    assert_eq!(read(&[0b0001_0100]), (1, 4));
    assert_eq!(read(&[0b1101_0001, 0b1001_0000, 0b0010_0000]), (5, 65793));
}
