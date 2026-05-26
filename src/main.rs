use private_vault::{
    api::{AppState, router},
    config::Config,
    files::FileService,
    replication::{ReplicationService},
    storage::Storage,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load();

    let storage = Storage::new(&config.storage_path).await?;

    let replication = ReplicationService::new(storage.clone(), config.peers);

    let file_service = FileService::new(storage.clone(), replication.clone());

    let state = AppState {
        node_id: config.node_id.clone(),
        storage,
        file_service,
    };

    let app = router(state);

    let address = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&address).await?;

    tracing::info!("starting node {} on {}", config.node_id, address);

    axum::serve(listener, app).await?;

    Ok(())
}
