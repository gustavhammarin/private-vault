use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;

use crate::{api::AppState, error::AppError, models::file::FileManifest};

pub async fn delete_manifest_test_only(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> anyhow::Result<StatusCode, AppError> {
    state
        .storage
        .delete_manifest(&file_id)
        .await
        .map_err(AppError::internal)?;

    Ok(StatusCode::OK)
}

pub async fn get_manifest(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> anyhow::Result<impl IntoResponse, AppError> {
    let manifest = state
        .storage
        .get_manifest(&file_id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::not_found("manifest not found"))?;

    Ok(Json(manifest))
}

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
