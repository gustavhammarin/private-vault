use std::sync::Arc;

use crate::{
    discovery::registry::PeerRegistry, models::peer::Peer, replication::{
        HttpPeerTransport, PeerTransport, placement::{Blake3ReplicaPlacement, ReplicaPlacement}, policy::ReplicationPolicy
    }, storage::Storage
};

#[derive(Debug, Clone)]
pub struct ReplicationService {
    pub(crate) storage: Storage,
    pub(crate) peer_registry: PeerRegistry,
    pub transport: Arc<dyn PeerTransport>,
    pub policy: ReplicationPolicy,
    pub placement: Arc<dyn ReplicaPlacement>,
}

impl ReplicationService {
    pub fn new(storage: Storage, peer_registry: PeerRegistry) -> Self {
        let transport = Arc::new(HttpPeerTransport::new());
        let policy = ReplicationPolicy::new(2);
        let placement = Arc::new(Blake3ReplicaPlacement);
        Self {
            storage,
            peer_registry,
            transport,
            policy,
            placement,
        }
    }
}
