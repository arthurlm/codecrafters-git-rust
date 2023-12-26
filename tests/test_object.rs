use std::io::BufReader;

use git_starter_rust::{object::GitObject, GitError};
use hex::ToHex;

#[test]
fn test_debug() {
    assert_eq!(
        format!("{:?}", GitObject::Blob(vec![1, 2, 3, 4])),
        "Blob([1, 2, 3, 4])"
    );
}

fn check_read_eq(input: &[u8], expected: GitObject) {
    let mut reader = BufReader::new(input);
    let val = GitObject::read(&mut reader).unwrap();
    assert_eq!(val, expected);
}

fn check_write_eq(input: GitObject, expected_data: &[u8], expected_code: &str) {
    let mut output = Vec::new();
    let code = input.write(&mut output).unwrap();
    assert_eq!(output, expected_data);
    assert_eq!(code.encode_hex::<String>(), expected_code);
}

fn check_err_eq(input: &[u8], expected: GitError) {
    let mut reader = BufReader::new(input);
    let err = GitObject::read(&mut reader).unwrap_err();
    assert_eq!(err, expected);
}

#[test]
fn test_read_blob() {
    check_read_eq(b"blob 5\0hello", GitObject::Blob(b"hello".to_vec()));
}

#[test]
fn test_write_blob() {
    check_write_eq(
        GitObject::Blob(b"world !".to_vec()),
        b"blob 7\0world !",
        "b172bdb8bda3a22be75a84d9c47f36fd2ead05c4",
    );
}

#[test]
fn test_read_invalid() {
    check_err_eq(b"", GitError::InvalidObjectHeader("missing header type"));
    check_err_eq(
        b"blob 5\0he",
        GitError::Io("failed to fill whole buffer".to_string()),
    );
}
