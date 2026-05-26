use crate::models::file::{FileManifest, FileSummary};
use anyhow::Result;
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait PeerTransport: Send + Sync + Debug {
    async fn has_chunk(&self, peer: &str, hash: &str) -> Result<bool>;
    async fn put_chunk(&self, peer: &str, hash: &str, data: Vec<u8>) -> Result<()>;
    async fn get_chunk(&self, peer: &str, hash: &str) -> Result<Option<Vec<u8>>>;
    async fn put_manifest(&self, peer: &str, manifest: &FileManifest) -> Result<()>;
    async fn get_manifest(&self, peer: &str, file_id: &str) -> Result<Option<FileManifest>>;
    async fn list_local_files(&self, peer: &str) -> Result<Vec<FileSummary>>;
}
