use std::sync::Arc;

use axum::{ extract::{Path, State}, http::{HeaderMap, HeaderValue}, response::{IntoResponse, Response}};

use crate::{api::AppState, error::AppError};

pub async fn get_chunk(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>
) -> anyhow::Result<Response, AppError> {

    let chunk = match state.storage.get_chunk(&hash).await {
        Ok(chunk) => chunk,
        Err(_) => {
            return Err(AppError::not_found("chunk not found"))
        },
    };

    let mut headers = HeaderMap::new();

    headers.insert(
        "content-type",
        HeaderValue::from_static("application/octet-stream")
    );

    Ok((headers, chunk).into_response())
}