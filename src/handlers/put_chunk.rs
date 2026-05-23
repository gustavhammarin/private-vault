use std::sync::Arc;

use axum::{body::Bytes, extract::{Path, State}};
use reqwest::StatusCode;

use crate::{api::AppState, error::AppError};

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