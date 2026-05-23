use crate::models::{ChunkMetadata, FileManifest};
use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct Storage {
    base_path: PathBuf,
    pool: SqlitePool,
}

impl Storage {
    pub async fn new(base_path: impl Into<PathBuf>) -> Result<Self> {
        let base_path = base_path.into();

        fs::create_dir_all(base_path.join("chunks")).await?;

        let database_url = database_url_for_path(&base_path);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        let storage = Self { base_path, pool };

        storage.migrate().await?;

        Ok(storage)
    }

    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                file_id TEXT PRIMARY KEY,
                file_name TEXT NOT NULL,
                size INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chunks (
                hash TEXT PRIMARY KEY,
                size INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_chunks (
                file_id TEXT NOT NULL,
                chunk_hash TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                size INTEGER NOT NULL,

                PRIMARY KEY (file_id, chunk_index),

                FOREIGN KEY (file_id) REFERENCES files(file_id),
                FOREIGN KEY (chunk_hash) REFERENCES chunks(hash)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

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

    pub async fn save_manifest(&self, manifest: &FileManifest) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO files (file_id, file_name, size)
            VALUES (?, ?, ?);
            "#,
        )
        .bind(&manifest.file_id)
        .bind(&manifest.file_name)
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

    pub async fn get_manifest(&self, file_id: &str) -> Result<FileManifest> {
        let file_row = sqlx::query(
            r#"
            SELECT file_id, file_name, size
            FROM files
            WHERE file_id = ?;
            "#,
        )
        .bind(file_id)
        .fetch_one(&self.pool)
        .await?;

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

        Ok(FileManifest {
            file_id: file_row.get::<String, _>("file_id"),
            file_name: file_row.get::<String, _>("file_name"),
            size: file_row.get::<i64, _>("size"),
            chunks,
        })
    }

    fn chunk_path(&self, hash: &str) -> PathBuf {
        self.base_path.join("chunks").join(hash)
    }
}

fn database_url_for_path(base_path: &Path) -> String {
    let db_path = base_path.join("vault.db");
    format!("sqlite://{}?mode=rwc", db_path.display())
}