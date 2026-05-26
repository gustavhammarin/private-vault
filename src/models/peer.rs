use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub node_id: String,
    pub adresses: PeerAdresses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerAdresses {
    pub http: String,
}
