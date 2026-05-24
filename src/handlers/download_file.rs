use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};

use crate::{api::AppState, error::AppError, models::FileManifest};

pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Response, AppError> {
    let manifest = get_manifest_or_fetch_from_peers(&state, &file_id).await?;

    repair_file_if_needed(&state, &manifest).await?;

    let mut result = Vec::with_capacity(manifest.size as usize);

    for chunk in &manifest.chunks {
        let data = state
            .storage
            .get_chunk(&chunk.hash)
            .await
            .map_err(|_| AppError::internal_msg(format!("missing chunk {}", chunk.hash)))?;
        result.extend_from_slice(&data);
    }

    let mut headers = HeaderMap::new();

    headers.insert(
        "content-type",
        HeaderValue::from_static("application/octet-stream"),
    );
    let content_disposition = format!("attachment; filename=\"{}\"", manifest.file_name);

    headers.insert(
        "content-disposition",
        HeaderValue::from_str(&content_disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    Ok((headers, result).into_response())
}

async fn repair_file_if_needed(state: &AppState, manifest: &FileManifest) -> Result<(), AppError> {
    let missing_chunks = state
        .storage
        .missing_chunks_for_file(manifest)
        .await
        .map_err(AppError::internal)?;

    if missing_chunks.is_empty() {
        return Ok(());
    }

    tracing::warn!(
        "file {} is missing {} chunks, trying repair",
        manifest.file_id,
        missing_chunks.len()
    );

    for chunk in missing_chunks {
        let data = state
            .replication
            .fetch_chunk_from_peers(&chunk.hash)
            .await
            .map_err(|_| {
                AppError::internal_msg(format!("could not repair missing chunk {}", chunk.hash))
            })?;

        state
            .storage
            .save_chunk(&chunk.hash, &data)
            .await
            .map_err(AppError::internal)?;

        tracing::info!(
            "repaired chunk {} for file {}",
            chunk.hash,
            manifest.file_id
        )
    }

    Ok(())
}

async fn get_manifest_or_fetch_from_peers(
    state: &AppState,
    file_id: &str,
) -> Result<FileManifest, AppError> {
    let existing = state
        .storage
        .get_manifest(file_id)
        .await
        .map_err(AppError::internal)?;

    if let Some(manifest) = existing {
        return Ok(manifest);
    }

    let manifest = state
        .replication
        .fetch_manifest_from_peers(file_id)
        .await
        .map_err(AppError::internal)?;

    state
        .storage
        .save_manifest(&manifest)
        .await
        .map_err(AppError::internal)?;

    Ok(manifest)
}
