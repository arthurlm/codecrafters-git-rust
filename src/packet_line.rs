use std::io;

use bytes::Bytes;

use crate::GitError;

#[derive(Debug, PartialEq, Eq)]
pub enum PacketLine {
    Command(Bytes),
    End,
}

impl PacketLine {
    pub fn command(data: &'static [u8]) -> Self {
        Self::Command(Bytes::from_static(data))
    }

    pub fn want(object_id: &str) -> Self {
        let data = format!("want {}", object_id);
        Self::Command(Bytes::from(data))
    }

    pub fn done() -> Self {
        Self::Command(Bytes::from_static(b"done"))
    }

    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            PacketLine::Command(data) => {
                write!(writer, "{:04x}", data.len() + 5)?;
                writer.write_all(data)?;
                writeln!(writer)?;
                Ok(())
            }
            PacketLine::End => write!(writer, "0000"),
        }
    }

    pub fn read<R: io::Read>(reader: &mut R) -> Result<Self, GitError> {
        let mut buf_size = [0_u8; 4];
        reader.read_exact(&mut buf_size)?;

        let data_len = usize::from_str_radix(std::str::from_utf8(&buf_size)?, 16)?;
        if data_len < 4 {
            Ok(Self::End)
        } else {
            let mut data = vec![0; data_len - 4];
            reader.read_exact(&mut data)?;

            Ok(Self::Command(Bytes::from(data)))
        }
    }
}
