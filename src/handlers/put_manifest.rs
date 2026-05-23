use std::sync::Arc;

use axum::{Json, extract::{Path, State}};
use reqwest::StatusCode;

use crate::{api::AppState, error::AppError, models::FileManifest};

pub async fn put_manifest(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
    Json(manifest): Json<FileManifest>,
) -> Result<StatusCode, AppError> {

    if manifest.file_id != file_id {
        return Err(AppError::bad_request_msg("manifest file id mismatch"));
    }

    state
        .storage
        .save_manifest(&manifest)
        .await
        .map_err(AppError::internal)?;

    Ok(StatusCode::CREATED)
}