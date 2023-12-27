use std::io::Read;

use git_starter_rust::fs_utils::*;

#[test]
fn test_rw() {
    let mut reader = read_compressed("8f6e02cd479bf43f6d853399037c447bf075988f").unwrap();
    let mut buf = Vec::with_capacity(512);
    reader.read_to_end(&mut buf).unwrap();

    assert_eq!(
        buf,
        b"commit 249\0tree 5d149ea6b58b6e64fd4889ec0c438fb8fae124cb\n\
          parent f7a087e57949c83b50d8b2b9d9566edef68cfcb3\n\
          author Arthur LE MOIGNE <arthur.lemoigne@gmail.com> 1703674545 +0100\n\
          committer Arthur LE MOIGNE <arthur.lemoigne@gmail.com> 1703675206 +0100\n\
          \n\
          hello commit\n"
    );
}
