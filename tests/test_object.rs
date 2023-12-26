use std::io::BufReader;

use git_starter_rust::{object::GitObject, GitError};

#[test]
fn test_debug() {
    assert_eq!(
        format!("{:?}", GitObject::Blob(vec![1, 2, 3, 4])),
        "Blob([1, 2, 3, 4])"
    );
}

fn check_eq(input: &[u8], expected: GitObject) {
    let mut reader = BufReader::new(input);
    let val = GitObject::read(&mut reader).unwrap();
    assert_eq!(val, expected);
}

fn check_err_eq(input: &[u8], expected: GitError) {
    let mut reader = BufReader::new(input);
    let err = GitObject::read(&mut reader).unwrap_err();
    assert_eq!(err, expected);
}

#[test]
fn test_read_blob() {
    check_eq(b"blob 5\0hello", GitObject::Blob(b"hello".to_vec()));
}

#[test]
fn test_read_invalid() {
    check_err_eq(b"", GitError::InvalidObjectHeader("missing header type"));
    check_err_eq(
        b"blob 5\0he",
        GitError::Io("failed to fill whole buffer".to_string()),
    );
}
