use std::io::BufReader;

use bytes::Bytes;
use git_starter_rust::{
    hash_code_text_to_array,
    object::{GitObject, GitTreeItem},
    GitError,
};

#[test]
fn test_debug() {
    assert_eq!(
        format!("{:?}", GitObject::Blob(Bytes::from_static(&[1, 2, 3, 4]))),
        "Blob(b\"\\x01\\x02\\x03\\x04\")"
    );
}

fn check_read_eq(input: &[u8], expected: GitObject) {
    let mut reader = BufReader::new(input);
    let val = GitObject::read(&mut reader).unwrap();
    assert_eq!(val, expected);
}

fn check_write_eq(input: GitObject, expected_data: &[u8], expected_code: &str) {
    let (code, output) = input.to_bytes_vec().unwrap();
    assert_eq!(output, expected_data);
    assert_eq!(hex::encode(code), expected_code);
}

fn check_err_eq(input: &[u8], expected: GitError) {
    let mut reader = BufReader::new(input);
    let err = GitObject::read(&mut reader).unwrap_err();
    assert_eq!(err, expected);
}

#[test]
fn test_read_blob() {
    check_read_eq(
        b"blob 5\0hello",
        GitObject::Blob(Bytes::from_static(b"hello")),
    );
}

#[test]
fn test_write_blob() {
    check_write_eq(
        GitObject::Blob(Bytes::from_static(b"world !")),
        b"blob 7\0world !",
        "b172bdb8bda3a22be75a84d9c47f36fd2ead05c4",
    );
}

#[test]
fn test_read_tree() {
    check_read_eq(
        include_bytes!("./data/simple-tree.bin"),
        build_expected_simple_tree(),
    );
}

#[test]
fn test_write_tree() {
    check_write_eq(
        build_expected_simple_tree(),
        include_bytes!("./data/simple-tree.bin"),
        "5110466bb52a33957176f544c5102765e867074b",
    );
}

#[test]
fn test_read_commit() {
    check_read_eq(
        include_bytes!("./data/simple-commit.bin"),
        build_expected_simple_commit(),
    );
}

#[test]
fn test_write_commit() {
    check_write_eq(
        build_expected_simple_commit(),
        include_bytes!("./data/simple-commit.bin"),
        "760d6c57bedca0f678995535b7882f52f92d31da",
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

fn build_expected_simple_tree() -> GitObject {
    GitObject::Tree(vec![
        GitTreeItem {
            mode: 0o100644,
            name: ".gitattributes".to_string(),
            hash_code: [
                23, 106, 69, 143, 148, 224, 234, 82, 114, 206, 103, 195, 107, 243, 11, 107, 233,
                202, 246, 35,
            ],
        },
        GitTreeItem {
            mode: 0o100644,
            name: ".gitignore".to_string(),
            hash_code: [
                55, 205, 33, 126, 116, 51, 86, 228, 30, 83, 151, 149, 91, 149, 16, 126, 175, 41,
                114, 119,
            ],
        },
        GitTreeItem {
            mode: 0o100644,
            name: "Cargo.lock".to_string(),
            hash_code: [
                9, 9, 141, 103, 3, 112, 147, 173, 205, 34, 174, 87, 72, 237, 235, 136, 114, 152,
                25, 227,
            ],
        },
        GitTreeItem {
            mode: 0o100644,
            name: "Cargo.toml".to_string(),
            hash_code: [
                47, 15, 148, 226, 221, 193, 10, 233, 172, 105, 254, 203, 158, 51, 113, 28, 102,
                129, 110, 69,
            ],
        },
        GitTreeItem {
            mode: 0o100644,
            name: "README.md".to_string(),
            hash_code: [
                142, 132, 161, 164, 130, 171, 142, 129, 234, 76, 185, 161, 98, 80, 167, 229, 229,
                183, 44, 141,
            ],
        },
        GitTreeItem {
            mode: 0o100644,
            name: "codecrafters.yml".to_string(),
            hash_code: [
                90, 223, 107, 145, 238, 213, 52, 104, 110, 77, 84, 88, 174, 255, 35, 2, 53, 232,
                203, 110,
            ],
        },
        GitTreeItem {
            mode: 0o40000,
            name: "src".to_string(),
            hash_code: [
                130, 180, 131, 22, 144, 167, 189, 105, 141, 13, 100, 104, 247, 17, 230, 27, 90,
                218, 61, 245,
            ],
        },
        GitTreeItem {
            mode: 0o40000,
            name: "tests".to_string(),
            hash_code: [
                206, 66, 221, 109, 130, 85, 214, 38, 45, 181, 239, 238, 198, 18, 112, 35, 87, 45,
                150, 34,
            ],
        },
        GitTreeItem {
            mode: 0o100755,
            name: "your_git.sh".to_string(),
            hash_code: [
                146, 162, 89, 8, 234, 154, 63, 46, 30, 85, 218, 89, 230, 228, 204, 239, 37, 221,
                189, 98,
            ],
        },
    ])
}

fn build_expected_simple_commit() -> GitObject {
    GitObject::Commit {
        tree: hash_code_text_to_array("e45ecd9e9fe4fcf69a6b35533afe57913090ce97"),
        parent: Some(hash_code_text_to_array(
            "74cc4ab80371ac64c33928d8c632e38de70a184f",
        )),
        author: Some("Arthur LE MOIGNE <arthur.lemoigne@gmail.com> 1703674545 +0100".to_string()),
        committer: Some(
            "Arthur LE MOIGNE <arthur.lemoigne@gmail.com> 1703675206 +0100".to_string(),
        ),
        message: "Add write-tree".to_string(),
    }
}
