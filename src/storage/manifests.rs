use anyhow::Result;
use sqlx::{types::Json, Row};

use crate::models::file::{ChunkMetadata, FileManifest, FileSummary};

use super::Storage;

impl Storage {
    pub async fn delete_manifest(&self, file_id: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
             DELETE FROM file_chunks
             WHERE file_id = ?;           
            "#,
        )
        .bind(file_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM files
            WHERE file_id = ?;
        "#,
        )
        .bind(file_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }
    pub async fn list_files(&self) -> anyhow::Result<Vec<FileSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT file_id, file_name, size
            FROM files
            ORDER BY created_at DESC;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let files = rows
            .into_iter()
            .map(|row| FileSummary {
                file_id: row.get("file_id"),
                file_name: row.get("file_name"),
                size: row.get("size"),
            })
            .collect();

        Ok(files)
    }
    pub async fn get_manifest(&self, file_id: &str) -> Result<Option<FileManifest>> {
        let file_row = sqlx::query(
            r#"
            SELECT file_id, file_name, content_type, replication_factor, target_peers, size
            FROM files
            WHERE file_id = ?;
            "#,
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(file_row) = file_row else {
            return Ok(None);
        };

        let chunk_rows = sqlx::query(
            r#"
            SELECT chunk_hash, chunk_index, size
            FROM file_chunks
            WHERE file_id = ?
            ORDER BY chunk_index ASC;
            "#,
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await?;

        let chunks = chunk_rows
            .into_iter()
            .map(|row| ChunkMetadata {
                hash: row.get::<String, _>("chunk_hash"),
                index: row.get::<i64, _>("chunk_index"),
                size: row.get::<i64, _>("size"),
            })
            .collect();

        Ok(Some(FileManifest {
            file_id: file_row.get::<String, _>("file_id"),
            file_name: file_row.get::<String, _>("file_name"),
            content_type: file_row.get::<Option<String>, _>("content_type"),
            replication_factor: file_row.get::<i32, _>("replication_factor"),
            target_peers: file_row.get::<Json<Vec<String>>, _>("target_peers").0,
            size: file_row.get::<i64, _>("size"),
            chunks,
        }))
    }
    pub async fn save_manifest(&self, manifest: &FileManifest) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO files (file_id, file_name, content_type, replication_factor, target_peers, size)
            VALUES (?, ?, ?, ?, ?, ?);
            "#,
        )
        .bind(&manifest.file_id)
        .bind(&manifest.file_name)
        .bind(&manifest.content_type)
        .bind(&manifest.replication_factor)
        .bind(Json(&manifest.target_peers))
        .bind(manifest.size)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM file_chunks
            WHERE file_id = ?;
            "#,
        )
        .bind(&manifest.file_id)
        .execute(&mut *tx)
        .await?;

        for chunk in &manifest.chunks {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO chunks (hash, size)
                VALUES (?, ?);
                "#,
            )
            .bind(&chunk.hash)
            .bind(chunk.size)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                INSERT INTO file_chunks (file_id, chunk_hash, chunk_index, size)
                VALUES (?, ?, ?, ?);
                "#,
            )
            .bind(&manifest.file_id)
            .bind(&chunk.hash)
            .bind(chunk.index)
            .bind(chunk.size)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
