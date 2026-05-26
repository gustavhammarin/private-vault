use serde::Serialize;

use crate::models::peer::Peer;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    #[serde(rename = "peerInfo")]
    pub peer_info: Peer,
    pub status: String,
}
