use std::sync::Arc;

use axum::{Json, extract::{Path, State}, response::IntoResponse};

use crate::{api::AppState, error::AppError};

pub async fn get_manifest(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>
) -> anyhow::Result<impl IntoResponse, AppError> {

    let manifest = state
        .storage
        .get_manifest(&file_id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::not_found("manifest not found"))?;

    Ok(Json(manifest))
}