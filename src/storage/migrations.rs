use anyhow::Result;

use super::Storage;

impl Storage {
    pub(super) async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                file_id TEXT PRIMARY KEY,
                file_name TEXT NOT NULL,
                content_type TEXT,
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
}
