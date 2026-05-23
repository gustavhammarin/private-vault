use crate::handlers::check_chunk::check_chunk;
use crate::handlers::download_file::download_file;
use crate::handlers::health::health;
use crate::handlers::put_chunk::put_chunk;
use crate::handlers::put_manifest::put_manifest;
use crate::handlers::upload_file::upload_file;

use crate::replication::ReplicationService;

use crate::storage::Storage;

use axum::{
    routing::{get, head, post, put}, Router,
};

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub node_id: String,
    pub storage: Storage,
    pub replication: ReplicationService,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/files", post(upload_file))
        .route("/files/{file_id}", get(download_file))
        .route("/chunks/{hash}", head(check_chunk).put(put_chunk))
        .route("/manifests/{file_id}", put(put_manifest))
        .with_state(Arc::new(state))
}
