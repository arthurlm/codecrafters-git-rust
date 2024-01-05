use std::io::BufReader;

use bytes::Bytes;
use git_starter_rust::packet_line::*;

#[test]
fn test_write() {
    fn check(p: PacketLine, expected: &[u8]) {
        let mut w = Vec::new();
        p.write(&mut w).unwrap();
        assert_eq!(w, expected);
    }

    check(PacketLine::End, b"0000");
    check(PacketLine::want("hello"), b"000fwant hello\n");
    check(PacketLine::want("world !"), b"0011want world !\n");
    check(PacketLine::done(), b"0009done\n");
}

#[test]
fn test_read() {
    fn check(buffer: &[u8], expected: PacketLine) {
        let mut reader = BufReader::new(buffer);
        let packet = PacketLine::read(&mut reader).unwrap();
        assert_eq!(packet, expected);
    }

    check(b"0000", PacketLine::End);
    check(b"0004", PacketLine::Command(Bytes::new()));
    check(b"0005\n", PacketLine::Command(Bytes::from_static(b"\n")));
    check(b"0010hello world\n", PacketLine::command(b"hello world\n"));
}
