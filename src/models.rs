use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub index: i64,
    pub hash: String,
    pub size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    #[serde(rename = "fileId")]
    pub file_id: String,

    #[serde(rename = "fileName")]
    pub file_name: String,

    pub size: i64,

    pub chunks: Vec<ChunkMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub size: i64
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterFileSummary {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub size: i64,
    #[serde(rename = "knownBy")]
    pub known_by: Vec<String>
}