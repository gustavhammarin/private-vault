use std::sync::Arc;

use anyhow::Result;
use axum::{extract::State, response::IntoResponse, Json};

use crate::{api::AppState, error::AppError};

pub async fn get_peer_info(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(state.peer_info.clone()))
}
