use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};

use bytes::{Buf, Bytes};
use tokio::{fs, try_join};

use crate::{
    fs_utils::read_compressed_at,
    hash_code_text_to_array,
    object::{GitObject, GitTreeItem},
    pack_file::unpack_into,
    packet_line::PacketLine,
    GitError,
};

pub async fn clone<P>(url: &str, dst: P) -> Result<(), GitError>
where
    P: AsRef<Path>,
{
    let dst = dst.as_ref();

    // Remove previous directory.
    let _ = fs::remove_dir_all(dst).await;

    // Prepare output dir.
    println!(">> Configuring new repository ...");
    fs::create_dir_all(dst).await?;
    fs::create_dir(dst.join(".git")).await?;

    try_join!(
        fs::create_dir(dst.join(".git/objects")),
        fs::create_dir_all(dst.join(".git/refs/heads")),
        fs::write(dst.join(".git/config"), ""),
        fs::write(dst.join(".git/description"), "empty repository"),
        fs::write(dst.join(".git/HEAD"), "ref: refs/heads/master\n"),
    )?;

    // List refs from remote repository.
    println!(">> Finding HEAD on remote server ...");
    let refs = InfoRef::list_for_repo(url).await?;

    // Find HEAD.
    let head = refs
        .into_iter()
        .find(|x| x.name == "HEAD")
        .ok_or(GitError::NoHead)?;

    // Configure HEAD
    println!(">> Configuring git repo ...");
    fs::write(dst.join(".git/refs/heads/master"), &head.object_id).await?;

    // Download it locally.
    println!(">> Downloading data ...");
    let mut reader = head.download(url).await?;

    // Unpack downloaded file
    println!(">> Unpacking data ...");
    unpack_into(&mut reader, dst)?;

    // Extract data from git database
    println!(">> Extracting data");
    extract_files_from_commit(&head.object_id, dst).await?;

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

    pub async fn download(&self, url: &str) -> Result<bytes::buf::Reader<Bytes>, GitError> {
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

        Ok(reader)
    }
}

async fn extract_files_from_commit<P>(commit_id: &str, dst: P) -> Result<(), GitError>
where
    P: AsRef<Path>,
{
    // Find object from git DB.
    let mut commit_data = read_compressed_at(hash_code_text_to_array(commit_id), &dst)?;
    let Ok(GitObject::Commit { tree, .. }) = GitObject::read(&mut commit_data) else {
        return Err(GitError::invalid_content(
            "Invalid object type, expected 'commit'",
        ));
    };

    // Find object from git DB.
    let mut tree_data = read_compressed_at(tree, &dst)?;
    let Ok(GitObject::Tree(items)) = GitObject::read(&mut tree_data) else {
        return Err(GitError::invalid_content(
            "Invalid object type, expected 'tree'",
        ));
    };

    // Extract tree
    let root = dst.as_ref().to_path_buf();
    extract_files_from_tree(items, root.clone(), root).await?;

    Ok(())
}

fn extract_files_from_tree(
    items: Vec<GitTreeItem>,
    dst: PathBuf,
    repo_root: PathBuf,
) -> Pin<Box<dyn Future<Output = Result<(), GitError>> + Send>> {
    Box::pin(async move {
        // For each tree item.
        for item in items {
            // Read and parse git object content.
            let mut item_data = read_compressed_at(item.hash_code, &repo_root)?;
            let obj_item = GitObject::read(&mut item_data)?;

            // Extract it to the file system.
            // TODO: handle file mode correctly 🤔.
            match obj_item {
                GitObject::Blob(obj_content) => {
                    fs::write(dst.join(item.name), obj_content).await?;
                }
                GitObject::Tree(sub_items) => {
                    let sub_dst = dst.join(item.name);
                    fs::create_dir(&sub_dst).await?;
                    extract_files_from_tree(sub_items, sub_dst, repo_root.clone()).await?;
                }
                // If we are here there is probably a bug 😭.
                GitObject::Commit { .. } => {
                    return Err(GitError::invalid_content(
                        "Cannot extract a commit as a file system object",
                    ))
                }
            }
        }

        Ok(())
    })
}
