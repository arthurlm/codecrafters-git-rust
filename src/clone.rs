use std::io;

use bytes::{Buf, Bytes};

use crate::{pack_file::PackFile, GitError};

pub async fn clone(url: &str) -> Result<(), GitError> {
    let refs = InfoRef::list_for_repo(url).await?;
    let head = refs
        .into_iter()
        .find(|x| x.name == "HEAD")
        .ok_or(GitError::NoHead)?;

    head.read(url).await?;
    Ok(())
}

#[derive(Debug)]
pub struct InfoRef {
    pub name: String,
    pub object_id: String,
    pub capabilities: Vec<String>,
}

impl InfoRef {
    pub async fn list_for_repo(url: &str) -> Result<Vec<Self>, GitError> {
        let client = reqwest::Client::new();
        let content = client
            .get(format!("{url}/info/refs?service=git-upload-pack"))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let mut output = Vec::new();

        for (idx, mut line) in content
            .lines()
            .skip(1) // Skip header
            .take_while(|x| *x != "0000") // Continue until we reach end tag
            .enumerate()
        {
            // Skip start sequence
            if idx == 0 {
                line = &line[4..];
            }

            // parse line like:
            // 004895dcfa3633004da0049d3d0fa03f80589cbcaf31 refs/heads/main\0multi_ack\n
            // 003c2cb58b79488a98d2721cea644875a8dd0026b115 refs/tags/v1.0\n
            let (obj_id, rem) = line
                .split_once(' ')
                .ok_or(GitError::InvalidObjectPayload("Missing object ID"))?;

            let (obj_name, capabilities) = match rem.split_once('\0') {
                Some((obj_name, rem)) => (
                    obj_name,
                    rem.split_whitespace().map(|x| x.to_string()).collect(),
                ),
                None => (rem, vec![]),
            };

            output.push(Self {
                name: obj_name.to_string(),
                object_id: obj_id[4..].to_string(),
                capabilities,
            });
        }

        Ok(output)
    }

    pub async fn read(&self, url: &str) -> Result<(), GitError> {
        let client = reqwest::Client::new();

        // Create git request
        let mut request_body = Vec::with_capacity(128);
        PacketLine::want(&self.object_id).write(&mut request_body)?;
        PacketLine::End.write(&mut request_body)?;
        PacketLine::done().write(&mut request_body)?;

        // Query server
        let content = client
            .post(format!("{url}/git-upload-pack"))
            .header("Content-Type", "application/x-git-upload-pack-request")
            .body(request_body)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let mut reader = content.reader();

        // Check first packet line is a NAK
        let command = PacketLine::read(&mut reader)?;
        if command != PacketLine::command(b"NAK\n") {
            return Err(GitError::Http("Bad response first packet line".to_string()));
        }

        // Read pack file
        PackFile::read(&mut reader)?;

        Ok(())
    }
}

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
