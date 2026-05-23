use std::sync::Arc;

use axum::{extract::{Path, State}, http::{HeaderMap, HeaderValue}, response::{IntoResponse, Response}};

use crate::{api::AppState, error::AppError};

pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Response, AppError> {

    let manifest = state
        .storage
        .get_manifest(&file_id)
        .await
        .map_err(|_| AppError::not_found("file not found"))?;
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