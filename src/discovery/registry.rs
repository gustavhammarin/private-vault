use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::peer::Peer;

#[derive(Debug, Clone)]
pub struct PeerRegistry {
    peers: Arc<RwLock<Vec<Peer>>>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn list(&self) -> Vec<Peer> {
        self.peers.read().await.clone()
    }

    pub async fn upsert(&self, peer: Peer) {
        let mut peers = self.peers.write().await;

        if let Some(existing) = peers.iter_mut().find(|p| p.node_id == peer.node_id) {
            *existing = peer;
        } else {
            peers.push(peer);
        }
    }
}
