use std::io::BufReader;

use git_starter_rust::clone::PacketLine;

#[test]
fn test_packet_line() {
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
fn test_parse_packet_line() {
    fn check(buffer: &[u8], expected: PacketLine) {
        let mut reader = BufReader::new(buffer);
        let packet = PacketLine::read(&mut reader).unwrap();
        assert_eq!(packet, expected);
    }

    check(b"0000", PacketLine::End);
    check(b"0004", PacketLine::Command(vec![]));
    check(b"0005\n", PacketLine::Command(vec![b'\n']));
    check(b"0010hello world\n", PacketLine::command(b"hello world\n"));
}
