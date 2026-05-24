use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{IntoResponse},
    Json,
};
use serde::Serialize;

use crate::{api::AppState, error::AppError, models::ChunkMetadata};

#[derive(Debug, Serialize)]
struct HealthResponse {
    #[serde(rename = "nodeId")]
    node_id: String,
    status: String,
}

pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(HealthResponse {
        node_id: state.node_id.clone(),
        status: "ok".to_string(),
    })
}

#[derive(Debug, Serialize)]
pub struct FileStatusResponse {
    #[serde(rename = "fileId")]
    file_id: String,
    #[serde(rename = "hasManifest")]
    has_manifest: bool,
    complete: bool,
    #[serde(rename = "missingChunks")]
    missing_chunks: Vec<ChunkMetadata>,
}

pub async fn file_status(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> anyhow::Result<impl IntoResponse, AppError> {
    
    let manifest = state
        .storage.get_manifest(&file_id)
        .await
        .map_err(AppError::internal)?;

    let Some(manifest) = manifest else {
        return Ok(Json(FileStatusResponse {
            file_id,
            has_manifest: false,
            complete: false,
            missing_chunks: Vec::new(),
        }));
    };
    let missing_chunks = state
        .storage
        .missing_chunks_for_file(&manifest)
        .await
        .map_err(AppError::internal)?;

    Ok(Json(FileStatusResponse {
        file_id,
        has_manifest: true,
        complete: missing_chunks.is_empty(),
        missing_chunks,
    }))
}
