mod error;
pub mod fs_utils;
pub mod header;
pub mod path_utils;
pub mod command {
    pub mod cat_file;
    pub mod commit_tree;
    pub mod hash_object;
    pub mod init;
    pub mod ls_tree;
    pub mod write_tree;
}
pub mod object;

pub use error::*;

pub type HashCode = [u8; 20];

pub fn hash_code_text_to_array(input: &str) -> HashCode {
    let mut array = [0_u8; 20];
    let data = hex::decode(input).unwrap();
    array.copy_from_slice(&data);
    array
}
