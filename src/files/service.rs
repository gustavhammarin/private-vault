use std::collections::HashMap;

use crate::{
    chunker::{chunk_bytes, to_metadata, DEFAULT_CHUNK_SIZE},
    files::schemas::FileStatusResponse,
    models::{
        cluster::ClusterFileSummary,
        file::{FileManifest, FileSummary},
    },
    replication::ReplicationService,
    storage::Storage,
};
use anyhow::Result;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileService {
    storage: Storage,
    replication: ReplicationService,
}
pub struct DownloadedFile {
    pub file_name: String,
    pub bytes: Vec<u8>,
}

impl FileService {
    pub fn new(storage: Storage, replication: ReplicationService) -> Self {
        Self {
            storage,
            replication,
        }
    }

    pub async fn upload_file(&self, file_name: String, bytes: Vec<u8>) -> Result<FileManifest> {
        let file_id = Uuid::new_v4().to_string();

        let chunks = chunk_bytes(&bytes, DEFAULT_CHUNK_SIZE);

        let mut manifest = FileManifest {
            file_id,
            file_name,
            size: 0,
            chunks: to_metadata(&chunks),
        };

        for chunk in chunks {
            self.storage.save_chunk(&chunk.hash, &chunk.data).await?;
            manifest.size += chunk.size;
        }

        self.storage.save_manifest(&manifest).await?;
        self.replication.replicate_file(&manifest).await;

        Ok(manifest)
    }

    pub async fn download_file(&self, file_id: &str) -> Result<DownloadedFile> {
        let manifest = self.get_manifest_or_fetch_from_peers(&file_id).await?;

        self.repair_file_if_needed(&manifest).await?;

        let mut result = Vec::with_capacity(manifest.size as usize);

        for chunk in &manifest.chunks {
            let data = self.storage.get_chunk(&chunk.hash).await?;
            result.extend_from_slice(&data);
        }
        Ok(DownloadedFile {
            file_name: manifest.file_name,
            bytes: result,
        })
    }

    pub async fn list_local_files(&self) -> Result<Vec<FileSummary>> {
        let files = self.storage.list_files().await?;

        Ok(files)
    }

    pub async fn list_cluster_files(&self, node_id: &str) -> Result<Vec<ClusterFileSummary>> {
        let mut files_by_id: HashMap<String, ClusterFileSummary> = HashMap::new();

        let local_files = self.storage.list_files().await?;

        for file in local_files {
            files_by_id.insert(
                file.file_id.clone(),
                ClusterFileSummary {
                    file_id: file.file_id,
                    file_name: file.file_name,
                    size: file.size,
                    known_by: vec![node_id.to_string()],
                },
            );
        }

        let cluster_files = self.replication.fetch_file_list_from_peers().await?;

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

        Ok(result)
    }

    pub async fn file_status(&self, file_id: &str) -> Result<FileStatusResponse> {
        let manifest = self.storage.get_manifest(&file_id).await?;

        let Some(manifest) = manifest else {
            return Ok(FileStatusResponse {
                file_id: file_id.to_string(),
                has_manifest: false,
                complete: false,
                missing_chunks: Vec::new(),
            });
        };
        let missing_chunks = self.storage.missing_chunks_for_file(&manifest).await?;

        Ok(FileStatusResponse {
            file_id: file_id.to_string(),
            has_manifest: true,
            complete: missing_chunks.is_empty(),
            missing_chunks,
        })
    }

    async fn get_manifest_or_fetch_from_peers(&self, file_id: &str) -> Result<FileManifest> {
        let existing = self.storage.get_manifest(file_id).await?;

        if let Some(manifest) = existing {
            return Ok(manifest);
        }

        let manifest = self.replication.fetch_manifest_from_peers(file_id).await?;

        self.storage.save_manifest(&manifest).await?;

        Ok(manifest)
    }

    async fn repair_file_if_needed(&self, manifest: &FileManifest) -> Result<()> {
        let missing_chunks = self.storage.missing_chunks_for_file(manifest).await?;

        if missing_chunks.is_empty() {
            return Ok(());
        }

        tracing::warn!(
            "file {} is missing {} chunks, trying repair",
            manifest.file_id,
            missing_chunks.len()
        );

        for chunk in missing_chunks {
            let data = self.replication.fetch_chunk_from_peers(&chunk.hash).await?;

            self.storage.save_chunk(&chunk.hash, &data).await?;

            tracing::info!(
                "repaired chunk {} for file {}",
                chunk.hash,
                manifest.file_id
            )
        }

        Ok(())
    }
}
