use std::sync::Arc;

use axum::extract::{Path, State};
use reqwest::StatusCode;

use crate::{api::AppState, error::AppError};

pub async fn delete_manifest_test_only(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>
) -> anyhow::Result<StatusCode, AppError> {

    state.storage.delete_manifest(&file_id).await.map_err(AppError::internal)?;

    Ok(StatusCode::OK)
}