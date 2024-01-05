use std::io::BufReader;

use git_starter_rust::{
    header::{GitObjectHeader, GitObjectHeaderType},
    GitError,
};

#[test]
fn test_debug() {
    assert_eq!(
        format!(
            "{:?}",
            GitObjectHeader {
                len: 10,
                r#type: GitObjectHeaderType::Blob
            }
        ),
        "GitObjectHeader { len: 10, type: Blob }"
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
    check_eq(
        b"blob 5\0hello",
        GitObjectHeader {
            len: 5,
            r#type: GitObjectHeaderType::Blob,
        },
    );
}

#[test]
fn test_read_tree() {
    check_eq(
        b"tree 269\0100644 .gitattributes",
        GitObjectHeader {
            len: 269,
            r#type: GitObjectHeaderType::Tree,
        },
    );
}

#[test]
fn test_read_commit() {
    check_eq(
        b"commit 9\0",
        GitObjectHeader {
            len: 9,
            r#type: GitObjectHeaderType::Commit,
        },
    );
}

#[test]
fn test_write_blob() {
    let mut out = Vec::new();
    GitObjectHeader {
        len: 50,
        r#type: GitObjectHeaderType::Blob,
    }
    .write(&mut out)
    .unwrap();
    assert_eq!(out, b"blob 50\0");
}

#[test]
fn test_write_tree() {
    let mut out = Vec::new();
    GitObjectHeader {
        len: 18,
        r#type: GitObjectHeaderType::Tree,
    }
    .write(&mut out)
    .unwrap();
    assert_eq!(out, b"tree 18\0");
}

#[test]
fn test_write_commit() {
    let mut out = Vec::new();
    GitObjectHeader {
        len: 897,
        r#type: GitObjectHeaderType::Commit,
    }
    .write(&mut out)
    .unwrap();
    assert_eq!(out, b"commit 897\0");
}

#[test]
fn test_read_invalid() {
    check_err_eq(b"", GitError::InvalidObjectHeader("missing header type"));
    check_err_eq(b"foo", GitError::InvalidObjectHeader("bad header len"));
    check_err_eq(b"blob bad", GitError::InvalidObjectHeader("bad header len"));
}
