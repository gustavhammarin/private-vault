mod models;
mod config;
mod chunker;
mod storage;
mod replication;
mod api;
mod error;
mod handlers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::Config::load();

    let storage = storage::Storage::new(&config.storage_path).await?;

    let replication = replication::ReplicationService::new(storage.clone(), config.peers);

    let state = api::AppState {
        node_id: config.node_id.clone(),
        storage,
        replication
    };

    let app = api::router(state);

    let address = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&address).await?;

    tracing::info!("starting node {} on {}", config.node_id, address);

    axum::serve(listener, app).await?;

    Ok(())
}
