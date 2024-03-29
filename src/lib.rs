pub mod clone;
mod error;
pub mod fs_utils;
pub mod header;
pub mod object;
pub mod pack_file;
pub mod packet_line;

pub use error::*;

pub type HashCode = [u8; 20];

pub fn hash_code_text_to_array(input: &str) -> HashCode {
    let mut array = [0_u8; 20];
    let data = hex::decode(input).expect("Invalid hash code received");
    array.copy_from_slice(&data);
    array
}
