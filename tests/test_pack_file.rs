use std::env;

use bytes::{Buf, Bytes};
use git_starter_rust::{
    pack_file::{read_object_pack_header, unpack_into, DeltaInstructionType},
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
        read_object_pack_header(&mut Bytes::from_static(data).reader()).unwrap()
    }

    assert_eq!(read(&[0b0000_0000]), (0, 0));
    assert_eq!(read(&[0b0001_0100]), (1, 4));
    assert_eq!(read(&[0b1101_0001, 0b1001_0000, 0b0010_0000]), (5, 65793));
}

#[test]
fn test_decode_delta_instr() {
    fn check(input: &[u8], expected: DeltaInstructionType) {
        let (rem, out) = DeltaInstructionType::from_bytes(input);
        assert_eq!(out, expected);
        assert_eq!(rem.len(), 0);
    }

    // Reserved
    check(&[0x00], DeltaInstructionType::Reserved);

    // Copy
    check(
        &[0b1011_0000, 0b1101_0001, 0b000_00001],
        DeltaInstructionType::Copy {
            offset: 0,
            size: 465,
        },
    );
    check(
        &[0b1010_1100, 0b1001_0000, 0b1111_0001, 0b0000_1110],
        DeltaInstructionType::Copy {
            offset: (0b1001_0000_usize << 16) | (0b1111_0001_usize << 24),
            size: 0b0000_1110_usize << 8,
        },
    );

    // Insert
    check(
        &[0b0111_0010],
        DeltaInstructionType::Insert { size: 0b0111_0010 },
    );
}
