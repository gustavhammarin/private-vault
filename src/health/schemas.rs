use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    pub status: String,
}
