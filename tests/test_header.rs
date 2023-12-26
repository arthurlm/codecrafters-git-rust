use std::io::BufReader;

use git_starter_rust::{header::GitObjectHeader, GitError};

#[test]
fn test_debug() {
    assert_eq!(
        format!("{:?}", GitObjectHeader::Blob { len: 10 }),
        "Blob { len: 10 }"
    );
}

fn check_eq(input: &[u8], expected: GitObjectHeader) {
    let mut reader = BufReader::new(input);
    let val = GitObjectHeader::read(&mut reader).unwrap();
    assert_eq!(val, expected);
}

fn check_err_eq(input: &[u8], expected: GitError) {
    let mut reader = BufReader::new(input);
    let err = GitObjectHeader::read(&mut reader).unwrap_err();
    assert_eq!(err, expected);
}

#[test]
fn test_read_blob() {
    check_eq(b"blob 5\0hello", GitObjectHeader::Blob { len: 5 });
}

#[test]
fn test_write_blob() {
    let mut out = Vec::new();
    GitObjectHeader::Blob { len: 50 }.write(&mut out).unwrap();
    assert_eq!(out, b"blob 50\0");
}

#[test]
fn test_read_invalid() {
    check_err_eq(b"", GitError::InvalidObjectHeader("missing header type"));
    check_err_eq(b"foo", GitError::InvalidObjectHeader("bad header type"));
    check_err_eq(b"blob bad", GitError::InvalidObjectHeader("bad header len"));
}
