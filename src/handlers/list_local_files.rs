use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};

use crate::{api::AppState, error::AppError};

pub async fn list_local_files(
    State(state): State<Arc<AppState>>,
) -> anyhow::Result<impl IntoResponse, AppError> {
    let files = state
        .storage
        .list_files()
        .await
        .map_err(AppError::internal)?;
    Ok(Json(files))
}
