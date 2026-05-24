use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;

use crate::{api::AppState, error::AppError};

pub async fn check_chunk(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
) -> Result<StatusCode, AppError> {
    let exists = state
        .storage
        .chunk_exists(&hash)
        .await
        .map_err(AppError::internal)?;
    if exists {
        Ok(StatusCode::OK)
    } else {
        Err(AppError::not_found("chunk not found"))
    }
}

pub async fn get_chunk(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
) -> anyhow::Result<Response, AppError> {
    let chunk = match state.storage.get_chunk(&hash).await {
        Ok(chunk) => chunk,
        Err(_) => return Err(AppError::not_found("chunk not found")),
    };

    let mut headers = HeaderMap::new();

    headers.insert(
        "content-type",
        HeaderValue::from_static("application/octet-stream"),
    );

    Ok((headers, chunk).into_response())
}

pub async fn put_chunk(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let calculated_hash = blake3::hash(&body).to_hex().to_string();

    if calculated_hash != hash {
        return Err(AppError::bad_request_msg("chunk hash mismatch"));
    }

    state
        .storage
        .save_chunk(&hash, &body)
        .await
        .map_err(AppError::internal)?;
    Ok(StatusCode::CREATED)
}
