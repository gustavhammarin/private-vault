use std::{collections::HashMap, sync::Arc};

use axum::{extract::State, response::IntoResponse, Json};

use crate::{api::AppState, error::AppError, models::ClusterFileSummary};

pub async fn list_cluster_files(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut files_by_id: HashMap<String, ClusterFileSummary> = HashMap::new();

    let local_files = state
        .storage
        .list_files()
        .await
        .map_err(AppError::internal)?;

    for file in local_files {
        files_by_id.insert(
            file.file_id.clone(),
            ClusterFileSummary {
                file_id: file.file_id,
                file_name: file.file_name,
                size: file.size,
                known_by: vec![state.node_id.clone()],
            },
        );
    }

    let cluster_files = state
        .replication
        .fetch_file_list_from_peers()
        .await
        .map_err(AppError::internal)?;

    for (peer, files) in cluster_files {
        for file in files {
            files_by_id
                .entry(file.file_id.clone())
                .and_modify(|existing| {
                    if !existing.known_by.contains(&peer) {
                        existing.known_by.push(peer.clone());
                    }
                })
                .or_insert(ClusterFileSummary {
                    file_id: file.file_id,
                    file_name: file.file_name,
                    size: file.size,
                    known_by: vec![peer.clone()],
                });
        }
    }

    let mut result: Vec<ClusterFileSummary> = files_by_id.into_values().collect();

    result.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    Ok(Json(result))
}
