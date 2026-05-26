use crate::{api::AppState, health::schemas::HealthResponse};
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(HealthResponse {
        peer_info: state.peer_info.clone(),
        status: "ok".to_string(),
    })
}
