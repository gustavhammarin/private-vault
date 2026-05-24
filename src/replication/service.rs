use crate::{replication::peer_client::PeerClient, storage::Storage};

#[derive(Debug, Clone)]
pub struct ReplicationService {
    pub(crate) storage: Storage,
    pub(crate) peers: Vec<String>,
    pub(crate) peer_client: PeerClient,
}

impl ReplicationService {
    pub fn new(storage: Storage, peers: Vec<String>) -> Self {
        Self {
            storage,
            peers,
            peer_client: PeerClient::new(),
        }
    }
}
