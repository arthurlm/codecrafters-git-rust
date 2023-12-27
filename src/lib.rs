mod error;
pub mod fs_utils;
pub mod header;
pub mod path_utils;
pub mod command {
    pub mod cat_file;
    pub mod hash_object;
    pub mod init;
    pub mod ls_tree;
    pub mod write_tree;
}
pub mod object;

pub use error::*;

pub type HashCode = [u8; 20];
