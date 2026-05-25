use anyhow::anyhow;
use reqwest::Client;
use crate::{models::file::{FileManifest, FileSummary}, replication::transport::PeerTransport};

#[derive(Debug, Clone)]
pub struct HttpPeerTransport {
    client: reqwest::Client,
}

impl HttpPeerTransport{
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl PeerTransport for HttpPeerTransport{
    
    async fn has_chunk(&self, peer: &str, hash: &str) -> anyhow::Result<bool> {
        let url = format!("{peer}/chunks/{hash}");
        let response = self.client.head(url).send().await?;

        if response.status().is_success() {
            return Ok(true);
        }

        if response.status().as_u16() == 404 {
            return Ok(false);
        }

        Err(anyhow!("unexpected status {}", response.status()))
    }

    async fn put_chunk(&self, peer: &str, hash: &str, data: Vec<u8>) -> anyhow::Result<()> {
        let url = format!("{peer}/chunks/{hash}");

        let response = self
            .client
            .put(url)
            .header("content-type", "application/octet-stream")
            .body(data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("peer returned status {}", response.status()));
        }

        Ok(())
    }

    async fn put_manifest(&self, peer: &str, manifest: &FileManifest) -> anyhow::Result<()> {
        let url = format!("{peer}/manifests/{}", manifest.file_id);

        let response = self.client.put(url).json(manifest).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("peer returned status {}", response.status()));
        }

        Ok(())
    }
    async fn get_chunk(&self, peer: &str, hash: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let url = format!("{peer}/chunks/{hash}");

        let response = self.client.get(url).send().await?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(anyhow!("peer returned status {}", response.status()));
        }

        let bytes = response.bytes().await?;
        Ok(Some(bytes.to_vec()))
    }
    async fn get_manifest(
        &self,
        peer: &str,
        file_id: &str,
    ) -> anyhow::Result<Option<FileManifest>> {
        let url = format!("{peer}/manifests/{file_id}");

        let response = self.client.get(url).send().await?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(anyhow!("peer returned status {}", response.status()));
        }

        Ok(Some(response.json::<FileManifest>().await?))
    }
    async fn list_local_files(&self, peer: &str) -> anyhow::Result<Vec<FileSummary>> {
        let url = format!("{peer}/files/local");

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("peer returned status {}", response.status()));
        }

        Ok(response.json::<Vec<FileSummary>>().await?)
    }
}