use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;

use crate::{api::AppState, error::AppError};

pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let (file_name, bytes) = read_file_from_multipart(&mut multipart).await?;

    let manifest = state
        .file_service
        .upload_file(file_name, bytes)
        .await
        .map_err(AppError::internal)?;

    Ok((StatusCode::CREATED, Json(manifest)))
}

async fn read_file_from_multipart(
    multipart: &mut Multipart,
) -> Result<(String, Vec<u8>), AppError> {
    let mut file_name = "uploaded-file".to_string();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(AppError::bad_request)?
    {
        if field.name() != Some("file") {
            continue;
        }

        if let Some(name) = field.file_name() {
            file_name = name.to_string();
        }

        let bytes = field.bytes().await.map_err(AppError::bad_request)?;
        return Ok((file_name, bytes.to_vec()));
    }

    Err(AppError::bad_request_msg("missing file field"))
}

pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> Result<Response, AppError> {
    let file = state
        .file_service
        .download_file(&file_id)
        .await
        .map_err(AppError::internal)?;

    let mut headers = HeaderMap::new();

    headers.insert(
        "content-type",
        HeaderValue::from_static("application/octet-stream"),
    );
    let content_disposition = format!("attachment; filename=\"{}\"", file.file_name);

    headers.insert(
        "content-disposition",
        HeaderValue::from_str(&content_disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
    );
    Ok((headers, file.bytes).into_response())
}

pub async fn list_local_files(
    State(state): State<Arc<AppState>>,
) -> anyhow::Result<impl IntoResponse, AppError> {
    let files = state
        .file_service
        .list_local_files()
        .await
        .map_err(AppError::internal)?;
    Ok(Json(files))
}

pub async fn list_cluster_files(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let files = state
        .file_service
        .list_cluster_files(&state.node_id)
        .await
        .map_err(AppError::internal)?;

    Ok(Json(files))
}

pub async fn file_status(
    State(state): State<Arc<AppState>>,
    Path(file_id): Path<String>,
) -> anyhow::Result<impl IntoResponse, AppError> {
    let status = state
        .file_service
        .file_status(&file_id)
        .await
        .map_err(AppError::internal)?;

    Ok(Json(status))
}
