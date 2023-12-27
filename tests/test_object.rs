use std::io::BufReader;

use git_starter_rust::{
    object::{GitObject, GitTreeItem},
    GitError,
};
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
fn test_read_tree() {
    check_read_eq(
        include_bytes!("./data/simple-tree.bin"),
        GitObject::Tree(vec![
            GitTreeItem {
                mode: 100644,
                name: ".gitattributes".to_string(),
                hash_code: [
                    23, 106, 69, 143, 148, 224, 234, 82, 114, 206, 103, 195, 107, 243, 11, 107,
                    233, 202, 246, 35,
                ],
            },
            GitTreeItem {
                mode: 100644,
                name: ".gitignore".to_string(),
                hash_code: [
                    55, 205, 33, 126, 116, 51, 86, 228, 30, 83, 151, 149, 91, 149, 16, 126, 175,
                    41, 114, 119,
                ],
            },
            GitTreeItem {
                mode: 100644,
                name: "Cargo.lock".to_string(),
                hash_code: [
                    9, 9, 141, 103, 3, 112, 147, 173, 205, 34, 174, 87, 72, 237, 235, 136, 114,
                    152, 25, 227,
                ],
            },
            GitTreeItem {
                mode: 100644,
                name: "Cargo.toml".to_string(),
                hash_code: [
                    47, 15, 148, 226, 221, 193, 10, 233, 172, 105, 254, 203, 158, 51, 113, 28, 102,
                    129, 110, 69,
                ],
            },
            GitTreeItem {
                mode: 100644,
                name: "README.md".to_string(),
                hash_code: [
                    142, 132, 161, 164, 130, 171, 142, 129, 234, 76, 185, 161, 98, 80, 167, 229,
                    229, 183, 44, 141,
                ],
            },
            GitTreeItem {
                mode: 100644,
                name: "codecrafters.yml".to_string(),
                hash_code: [
                    90, 223, 107, 145, 238, 213, 52, 104, 110, 77, 84, 88, 174, 255, 35, 2, 53,
                    232, 203, 110,
                ],
            },
            GitTreeItem {
                mode: 40000,
                name: "src".to_string(),
                hash_code: [
                    130, 180, 131, 22, 144, 167, 189, 105, 141, 13, 100, 104, 247, 17, 230, 27, 90,
                    218, 61, 245,
                ],
            },
            GitTreeItem {
                mode: 40000,
                name: "tests".to_string(),
                hash_code: [
                    206, 66, 221, 109, 130, 85, 214, 38, 45, 181, 239, 238, 198, 18, 112, 35, 87,
                    45, 150, 34,
                ],
            },
            GitTreeItem {
                mode: 100755,
                name: "your_git.sh".to_string(),
                hash_code: [
                    146, 162, 89, 8, 234, 154, 63, 46, 30, 85, 218, 89, 230, 228, 204, 239, 37,
                    221, 189, 98,
                ],
            },
        ]),
    );
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
