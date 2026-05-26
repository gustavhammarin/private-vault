use crate::chunks::{check_chunk, get_chunk, put_chunk};
use crate::files::{
    download_file, file_status, list_cluster_files, list_local_files, upload_file, FileService,
};
use crate::health::health;
use crate::manifests::{delete_manifest_test_only, get_manifest, put_manifest};
use crate::storage::Storage;

use axum::extract::DefaultBodyLimit;
use axum::{
    routing::{get, head, post, put},
    Router,
};

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub node_id: String,
    pub storage: Storage,
    pub file_service: FileService,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/files", post(upload_file).layer(DefaultBodyLimit::max(100 * 1024 * 1024)))
        .route("/files/local", get(list_local_files))
        .route("/files/all", get(list_cluster_files))
        .route("/files/{file_id}", get(download_file))
        .route("/files/{file_id}/status", get(file_status))
        .route(
            "/chunks/{hash}",
            head(check_chunk).get(get_chunk).put(put_chunk),
        )
        .route(
            "/manifests/{file_id}",
            put(put_manifest)
                .get(get_manifest)
                .delete(delete_manifest_test_only),
        )
        .with_state(Arc::new(state))
}
