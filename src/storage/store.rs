use std::path::{Path, PathBuf};

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tokio::fs;

#[derive(Debug, Clone)]
pub struct Storage {
    pub(super) base_path: PathBuf,
    pub(super) pool: SqlitePool,
}

impl Storage {
    pub async fn new(base_path: impl Into<PathBuf>) -> anyhow::Result<Self> {
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
}

fn database_url_for_path(base_path: &Path) -> String {
    let db_path = base_path.join("vault.db");
    format!("sqlite://{}?mode=rwc", db_path.display())
}
