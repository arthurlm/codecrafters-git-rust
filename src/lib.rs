mod error;
pub mod fs_utils;
pub mod header;
pub mod object;
pub mod path_utils;

pub use error::*;

pub type HashCode = [u8; 20];

pub fn hash_code_text_to_array(input: &str) -> HashCode {
    let mut array = [0_u8; 20];
    let data = hex::decode(input).unwrap();
    array.copy_from_slice(&data);
    array
}
