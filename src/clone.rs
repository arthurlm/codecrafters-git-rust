use std::path::Path;

use bytes::Buf;
use tokio::{fs, try_join};

use crate::{pack_file::unpack_into, packet_line::PacketLine, GitError};

pub async fn clone<P: AsRef<Path> + Clone>(url: &str, dst: P) -> Result<(), GitError> {
    let dst = dst.as_ref();

    // Remove previous directory.
    let _ = fs::remove_dir_all(dst).await;

    // Prepare output dir.
    fs::create_dir_all(dst).await?;
    fs::create_dir(dst.join(".git")).await?;

    try_join!(
        fs::create_dir(dst.join(".git/objects")),
        fs::create_dir(dst.join(".git/refs")),
        fs::write(dst.join(".git/config"), ""),
        fs::write(dst.join(".git/description"), "empty repository"),
    )?;

    // List refs from remote repository.
    let refs = InfoRef::list_for_repo(url).await?;

    // Find HEAD.
    let head = refs
        .into_iter()
        .find(|x| x.name == "HEAD")
        .ok_or(GitError::NoHead)?;

    // Download it locally.
    head.download_into(url, dst).await?;
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

    pub async fn download_into<P: AsRef<Path> + Clone>(
        &self,
        url: &str,
        dst: P,
    ) -> Result<(), GitError> {
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
        unpack_into(&mut reader, dst)?;

        Ok(())
    }
}
