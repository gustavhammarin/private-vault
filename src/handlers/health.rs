use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;

use crate::api::AppState;

#[derive(Debug, Serialize)]
struct HealthResponse {
    #[serde(rename = "nodeId")]
    node_id: String,
    status: String,
}

pub async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(HealthResponse {
        node_id: state.node_id.clone(),
        status: "ok".to_string(),
    })
}