use crate::{models::file::FileManifest, replication::service::ReplicationService};

impl ReplicationService {
    pub async fn replicate_file(&self, manifest: &mut FileManifest) {
        let peers = self.peer_registry.list().await;
        
        if peers.is_empty() {
            tracing::info!("no peers configured, skipping replication");
            return;
        }

        let target_peers =
            self.placement
                .target_peers_for_file(&manifest.file_id, &peers, &self.policy);
        manifest.target_peers = target_peers.clone();
        manifest.replication_factor = self.policy.replication_factor as i32;

        for target_node_id in target_peers {
            let Some(peer) = peers.iter().find(|peer| peer.node_id == target_node_id) else {
                tracing::warn!("target node {} is not currently resolved", target_node_id);
                continue;
            };

            let peer_url = &peer.adresses.http;
            
            tracing::info!("replicating file {} to {}", manifest.file_id, peer.node_id);

            for chunk in &manifest.chunks {
                if let Err(err) = self.replicate_chunk_to_peer(&peer_url, &chunk.hash).await {
                    tracing::error!(
                        "failed to replicate chunk {} to {}: {:?}",
                        chunk.hash,
                        peer.node_id,
                        err
                    );
                }
            }

            if let Err(err) = self.transport.put_manifest(&peer.adresses.http, manifest).await {
                tracing::error!(
                    "failed to replicate manifest {} to {}: {:?}",
                    manifest.file_id,
                    peer.node_id,
                    err
                );
            }
        }
    }
    async fn replicate_chunk_to_peer(&self, peer_url: &str, hash: &str) -> anyhow::Result<()> {
        if self.transport.has_chunk(peer_url, hash).await.unwrap_or(false) {
            tracing::info!("peer {} already has chunk {}", peer_url, hash);
            return Ok(());
        }

        let data = self.storage.get_chunk(hash).await?;
        self.transport.put_chunk(peer_url, hash, data).await
    }
}
