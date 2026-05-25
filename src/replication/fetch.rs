use anyhow::anyhow;

use crate::{models::file::{FileManifest, FileSummary}};

use super::service::ReplicationService;

impl ReplicationService {
    pub async fn fetch_chunk_from_peers(&self, hash: &str) -> anyhow::Result<Vec<u8>> {
        for peer in &self.peers {
            tracing::info!("trying to fetch chunk {} from {}", hash, peer);

            let Some(data) = self.fetch_chunk_from_peer(peer, hash).await else {
                continue;
            };

            return Ok(data);
        }

        Err(anyhow!("chunk {} not found on any peer", hash))
    }

    async fn fetch_chunk_from_peer(&self, peer: &str, hash: &str) -> Option<Vec<u8>> {
        let data = match self.transport.get_chunk(peer, hash).await {
            Ok(Some(data)) => data,
            Ok(None) => {
                tracing::warn!("peer {} does not have chunk {}", peer, hash);
                return None;
            }
            Err(err) => {
                tracing::warn!("failed to fetch chunk {} from {}: {:?}", hash, peer, err);
                return None;
            }
        };

        let calculated_hash = blake3::hash(&data).to_hex().to_string();

        if calculated_hash != hash {
            tracing::warn!("peer {} returned invalid data for chunk {}", peer, hash);
            return None;
        }

        tracing::info!("fetched chunk {} from {}", hash, peer);
        Some(data)
    }

    pub async fn fetch_manifest_from_peers(&self, file_id: &str) -> anyhow::Result<FileManifest> {
        for peer in &self.peers {
            match self.transport.get_manifest(peer, file_id).await {
                Ok(Some(manifest)) => {
                    tracing::info!("found manifest {} from peer {}", file_id, peer);
                    return Ok(manifest);
                }
                Ok(None) => {
                    tracing::warn!("peer {} does not have manifest {}", peer, file_id);
                }
                Err(err) => {
                    tracing::warn!(
                        "failed to fetch manifest {} from {}: {:?}",
                        file_id,
                        peer,
                        err
                    );
                }
            }
        }

        Err(anyhow!("manifest {} not found at any peer", file_id))
    }

    pub async fn fetch_file_list_from_peers(
        &self,
    ) -> anyhow::Result<Vec<(String, Vec<FileSummary>)>> {
        let mut result = Vec::new();

        for peer in &self.peers {
            match self.transport.list_local_files(peer).await {
                Ok(files) => {
                    result.push((peer.clone(), files));
                }
                Err(err) => {
                    tracing::warn!("failed to fetch file list from peer {}: {:?}", peer, err);
                }
            }
        }

        Ok(result)
    }
}
