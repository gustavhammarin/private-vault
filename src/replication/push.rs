use crate::{models::file::FileManifest, replication::{service::ReplicationService}};

impl ReplicationService {
    pub async fn replicate_file(&self, manifest: &FileManifest) {
        if self.peers.is_empty() {
            tracing::info!("no peers configured, skipping replication");
            return;
        }

        let target_peers = self.placement.target_peers_for_file(&manifest.file_id, &self.peers, &self.policy);

        for peer in target_peers {
            tracing::info!("replicating file {} to {}", manifest.file_id, peer);

            for chunk in &manifest.chunks {
                if let Err(err) = self.replicate_chunk_to_peer(&peer, &chunk.hash).await {
                    tracing::error!(
                        "failed to replicate chunk {} to {}: {:?}",
                        chunk.hash,
                        peer,
                        err
                    );
                }
            }

            if let Err(err) = self.transport.put_manifest(&peer, manifest).await {
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
        if self
            .transport
            .has_chunk(peer, hash)
            .await
            .unwrap_or(false)
        {
            tracing::info!("peer {} already has chunk {}", peer, hash);
            return Ok(());
        }

        let data = self.storage.get_chunk(hash).await?;
        self.transport.put_chunk(peer, hash, data).await
    }
}
