use anyhow::anyhow;
use reqwest::Client;

use crate::{
    models::{FileManifest, FileSummary},
    storage::Storage,
};

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

    async fn replicate_chunk_to_peer(&self, peer: &str, hash: &str) -> anyhow::Result<()> {
        if self.peer_has_chunk(peer, hash).await.unwrap_or(false) {
            tracing::info!("peer {} already has chunk {}", peer, hash);
            return Ok(());
        }

        let data = self.storage.get_chunk(hash).await?;
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

    async fn peer_has_chunk(&self, peer: &str, hash: &str) -> anyhow::Result<bool> {
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

    async fn replicate_manifest_to_peer(
        &self,
        peer: &str,
        manifest: &FileManifest,
    ) -> anyhow::Result<()> {
        let url = format!("{peer}/manifests/{}", manifest.file_id);

        let response = self.client.put(url).json(manifest).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("peer returned status {}", response.status()));
        }
        Ok(())
    }

    pub async fn fetch_chunk_from_peers(&self, hash: &str) -> anyhow::Result<Vec<u8>> {
        for peer in &self.peers {
            let url = format!("{peer}/chunks/{hash}");

            tracing::info!("trying to fetch chunk {} from {}", hash, peer);

            let response = match self.client.get(&url).send().await {
                Ok(response) => response,
                Err(err) => {
                    tracing::warn!("failed to contact peer {}: {:?}", peer, err);
                    continue;
                }
            };

            if response.status().as_u16() == 404 {
                tracing::warn!("peer {} does not have chunk {}", peer, hash);
                continue;
            }

            if !response.status().is_success() {
                tracing::warn!(
                    "peer {} returned status {} for chunk {}",
                    peer,
                    response.status(),
                    hash
                );
                continue;
            }

            let bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => {
                    tracing::warn!("failed to read chunk {} from peer {}", hash, peer);
                    continue;
                }
            };

            let calculated_hash = blake3::hash(&bytes).to_hex().to_string();

            if calculated_hash != hash {
                tracing::warn!("peer {} returned invalid data for chunk {}", peer, hash);
                continue;
            }

            tracing::info!("fetched chunk {} from {}", hash, peer);
            return Ok(bytes.to_vec());
        }

        Err(anyhow!("chunk {} not found on any peer", hash))
    }

    pub async fn fetch_manifest_from_peers(&self, file_id: &str) -> anyhow::Result<FileManifest> {
        for peer in &self.peers {
            let url = format!("{peer}/manifests/{file_id}");

            let response = match self.client.get(&url).send().await {
                Ok(response) => response,
                Err(_) => {
                    tracing::warn!("failed to contact peer: {}", peer);
                    continue;
                }
            };

            if response.status().as_u16() == 404 {
                tracing::warn!("peer {} does not have manifest {}", peer, file_id);
                continue;
            }

            if !response.status().is_success() {
                tracing::warn!(
                    "peer {} returned status {} for manifest {}",
                    peer,
                    response.status(),
                    file_id
                );
                continue;
            }

            let manifest = match response.json::<FileManifest>().await {
                Ok(manifest) => manifest,
                Err(_) => {
                    tracing::warn!("failed to read response body from peer {}", peer);
                    continue;
                }
            };

            tracing::info!("Found manifest {} from peer {}", file_id, peer);
            return Ok(manifest);
        }

        Err(anyhow!("manifest {} not found at any peer", file_id))
    }

    pub async fn fetch_file_list_from_peers(
        &self,
    ) -> anyhow::Result<Vec<(String, Vec<FileSummary>)>> {
        let mut result = Vec::new();

        for peer in &self.peers {
            let url = format!("{peer}/files/local");

            let response = match self.client.get(&url).send().await {
                Ok(response) => response,
                Err(_) => {
                    tracing::warn!("failed to fetch file list from peer {}", peer);
                    continue;
                }
            };
            if !response.status().is_success() {
                tracing::warn!(
                    "peer {} returned status {} when listing files",
                    peer,
                    response.status()
                );
                continue;
            }

            let files = match response.json::<Vec<FileSummary>>().await {
                Ok(files) => files,
                Err(err) => {
                    tracing::warn!("peer {} returned invalid file list: {:?}", peer, err);
                    continue;
                }
            };

            result.push((peer.clone(), files));
        }
        Ok(result)
    }
}
