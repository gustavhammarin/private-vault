use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

use crate::models::file::{ChunkMetadata, FileManifest};

use super::Storage;

impl Storage {
    pub async fn save_chunk(&self, hash: &str, data: &[u8]) -> Result<()> {
        let path = self.chunk_path(hash);

        if !fs::try_exists(&path).await? {
            fs::write(&path, data).await?;
        }

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO chunks (hash, size)
            VALUES (?, ?);
            "#,
        )
        .bind(hash)
        .bind(data.len() as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_chunk(&self, hash: &str) -> Result<Vec<u8>> {
        let data = fs::read(self.chunk_path(hash)).await?;
        Ok(data)
    }

    pub async fn chunk_exists(&self, hash: &str) -> Result<bool> {
        Ok(fs::try_exists(self.chunk_path(hash)).await?)
    }
    fn chunk_path(&self, hash: &str) -> PathBuf {
        self.base_path.join("chunks").join(hash)
    }

    pub async fn missing_chunks_for_file(
        &self,
        manifest: &FileManifest,
    ) -> anyhow::Result<Vec<ChunkMetadata>> {
        let mut missing = Vec::new();

        for chunk in &manifest.chunks {
            let exists = self.chunk_exists(&chunk.hash).await?;

            if !exists {
                missing.push(chunk.clone());
            }
        }

        Ok(missing)
    }
}
