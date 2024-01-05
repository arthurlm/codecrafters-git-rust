use std::io::Read;

use git_starter_rust::{fs_utils::*, hash_code_text_to_array};

#[test]
fn test_rw() {
    let mut reader = read_compressed(hash_code_text_to_array(
        "fa6eb0c05a11f886f3d5e15f7c1dc794428b9228",
    ))
    .unwrap();
    let mut buf = Vec::with_capacity(512);
    reader.read_to_end(&mut buf).unwrap();

    assert_eq!(
        buf,
        b"commit 258\x00tree 89ad93569b2e3de0e8b6ce13f4b9bf9bfea055e6\n\
          parent 46e8640151849752d25446e9273b4016d27de2f0\n\
          author Arthur LE MOIGNE <arthur.lemoigne@gmail.com> 1703679320 +0100\n\
          committer Arthur LE MOIGNE <arthur.lemoigne@gmail.com> 1703679320 +0100\n\
          \n\
          Add tests on fs utils\n"
    );
}
