use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterFileSummary {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub size: i64,
    #[serde(rename = "knownBy")]
    pub known_by: Vec<String>,
}
