use std::env;

use bytes::{Buf, Bytes};
use git_starter_rust::{
    pack_file::{read_object_size, unpack_into},
    GitError,
};

fn unpack_static(data: &'static [u8]) {
    unpack_into(&mut Bytes::from_static(data).reader(), env::temp_dir()).unwrap()
}

fn unpack_static_err(data: &'static [u8]) -> GitError {
    unpack_into(&mut Bytes::from_static(data).reader(), env::temp_dir()).unwrap_err()
}

#[test]
fn test_invalid_parse() {
    assert_eq!(
        unpack_static_err(b""),
        GitError::io("failed to fill whole buffer")
    );
    assert_eq!(
        unpack_static_err(b"PUCK"),
        GitError::invalid_content("Invalid magic PACK")
    );
    assert_eq!(
        unpack_static_err(b"PACK"),
        GitError::io("failed to fill whole buffer")
    );
    assert_eq!(
        unpack_static_err(b"PACK\0\0\0\0"),
        GitError::invalid_content("Invalid PACK version")
    );
}

#[test]
fn test_valid_parse() {
    unpack_static(include_bytes!("./data/sqlite-rust.pack"));
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
