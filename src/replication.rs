use anyhow::{anyhow};
use reqwest::Client;

use crate::{models::FileManifest, storage::Storage};

#[derive(Debug, Clone)]
pub struct ReplicationService {
    storage: Storage,
    peers: Vec<String>,
    client: Client,
}

impl ReplicationService {
    pub fn new(storage: Storage, peers: Vec<String>) -> Self {
        Self {
            storage,
            peers,
            client: Client::new(),
        }
    }

    pub async fn replicate_file(&self, manifest: &FileManifest) {
        if self.peers.is_empty() {
            tracing::info!("no peers configured, skipping replication");
            return;
        }

        for peer in &self.peers {
            tracing::info!("replicating file {} to {}", manifest.file_id, peer);

            for chunk in &manifest.chunks {
                if let Err(err) = self.replicate_chunk_to_peer(peer, &chunk.hash).await {
                    tracing::error!(
                        "failed to replicate chunk {} to {}: {:?}",
                        chunk.hash,
                        peer,
                        err
                    );
                }
            }

            if let Err(err) = self.replicate_manifest_to_peer(peer, manifest).await {
                tracing::error!(
                    "failed to replicate manifest {} to {}: {:?}",
                    manifest.file_id,
                    peer,
                    err
                );
            }
        }
    }

    async fn replicate_chunk_to_peer(&self, peer: &str, hash:&str) -> anyhow::Result<()> {
        if self.peer_has_chunk(peer, hash).await.unwrap_or(false) {
            tracing::info!("peer {} already has chunk {}", peer, hash);
            return Ok(())
        }

        let data = self.storage.get_chunk(hash).await?;
        let url = format!("{peer}/chunks/{hash}");

        let response = self.client
            .put(url)
            .header("content-type", "application/octet-stream")
            .body(data)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(anyhow!("peer returned status {}", response.status()))
        }
        Ok(())
    }

    async fn peer_has_chunk(&self, peer: &str, hash: &str) -> anyhow::Result<bool> {
        let url = format!("{peer}/chunks/{hash}");

        let response = self.client.head(url).send().await?;

        if response.status().is_success() {
            return Ok(true)
        }

        if response.status().as_u16() == 404 {
            return Ok(false);
        }
        Err(anyhow!("unexpected status {}", response.status()))
    }

    async fn replicate_manifest_to_peer(
        &self,
        peer: &str,
        manifest: &FileManifest,
    ) -> anyhow::Result<()> {

        let url = format!("{peer}/manifests/{}", manifest.file_id);

        let response = self.client.put(url).json(manifest).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("peer returned status {}", response.status()))
        }
        Ok(())
    }
}
