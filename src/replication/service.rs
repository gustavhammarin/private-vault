use std::sync::Arc;

use crate::{replication::{PeerTransport}, storage::Storage};

#[derive(Debug, Clone)]
pub struct ReplicationService {
    pub(crate) storage: Storage,
    pub(crate) peers: Vec<String>,
    pub transport: Arc<dyn PeerTransport>
}

impl ReplicationService {
    pub fn new(storage: Storage, peers: Vec<String>, transport: Arc<dyn PeerTransport>) -> Self {
        Self {
            storage,
            peers,
            transport
        }
    }
}
