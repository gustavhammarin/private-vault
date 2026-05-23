use std::sync::Arc;

use axum::extract::{Path, State};
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