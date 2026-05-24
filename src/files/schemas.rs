use serde::Serialize;

use crate::models::file::ChunkMetadata;

#[derive(Debug, Serialize)]
pub struct FileStatusResponse {
    #[serde(rename = "fileId")]
    pub file_id: String,
    #[serde(rename = "hasManifest")]
    pub has_manifest: bool,
    pub complete: bool,
    #[serde(rename = "missingChunks")]
    pub missing_chunks: Vec<ChunkMetadata>,
}
