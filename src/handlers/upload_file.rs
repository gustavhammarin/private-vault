use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use uuid::Uuid;

use crate::{
    api::AppState,
    chunker::{chunk_bytes, to_metadata, DEFAULT_CHUNK_SIZE},
    error::AppError,
    models::FileManifest,
};

pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let _ = multipart;

    let mut file_name = "uploaded-file".to_string();

    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(AppError::bad_request)?
    {
        if field.name() == Some("file") {
            if let Some(name) = field.file_name() {
                file_name = name.to_string();
            }

            let bytes = field.bytes().await.map_err(AppError::bad_request)?;
            file_bytes = Some(bytes.to_vec());
            break;
        }
    }

    let file_bytes = file_bytes.ok_or_else(|| AppError::bad_request_msg("missing file field"))?;

    let file_id = Uuid::new_v4().to_string();

    let chunks = chunk_bytes(&file_bytes, DEFAULT_CHUNK_SIZE);

    let mut manifest = FileManifest {
        file_id,
        file_name,
        size: 0,
        chunks: to_metadata(&chunks),
    };

    for chunk in chunks {
        state
            .storage
            .save_chunk(&chunk.hash, &chunk.data)
            .await
            .map_err(AppError::internal)?;
        manifest.size += chunk.size;
    }

    state
        .storage
        .save_manifest(&manifest)
        .await
        .map_err(AppError::internal)?;
    state.replication.replicate_file(&manifest).await;
    Ok((StatusCode::CREATED, Json(manifest)))
}
