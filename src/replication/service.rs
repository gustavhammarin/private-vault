use std::sync::Arc;

use crate::{replication::{HttpPeerTransport, PeerTransport, placement::{ Blake3ReplicaPlacement, ReplicaPlacement}, policy::{ReplicationPolicy}}, storage::Storage};

#[derive(Debug, Clone)]
pub struct ReplicationService {
    pub(crate) storage: Storage,
    pub(crate) peers: Vec<String>,
    pub transport: Arc<dyn PeerTransport>,
    pub policy: ReplicationPolicy,
    pub placement: Arc<dyn ReplicaPlacement>
}

impl ReplicationService {
    pub fn new(storage: Storage, peers: Vec<String>) -> Self {
        let transport = Arc::new(HttpPeerTransport::new());
        let policy = ReplicationPolicy::new(2);
        let placement = Arc::new(Blake3ReplicaPlacement);
        Self {
            storage,
            peers,
            transport,
            policy,
            placement
        }
    }
}
