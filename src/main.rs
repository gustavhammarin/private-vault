use private_vault::{
    api::{AppState, router}, config::Config, discovery::{ PeerDiscoveryService, registry::PeerRegistry}, files::FileService, models::peer::{Peer, PeerAdresses}, replication::ReplicationService, storage::Storage
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load();

    let storage = Storage::new(&config.storage_path).await?;

    let peer_registry = PeerRegistry::new();

    let replication = ReplicationService::new(storage.clone(), peer_registry.clone());

    let file_service = FileService::new(storage.clone(), replication.clone());

    let state = AppState {
        peer_info: Peer {
            node_id: config.node_id.clone(),
            adresses: PeerAdresses {
                http: format!("http://{}:{}", config.node_id, config.port),
            },
        },
        storage,
        file_service,
    };

    let discovery = PeerDiscoveryService::new(config.peers_base_urls.clone());
    let discovery_registry = peer_registry.clone();

    tokio::spawn(async move {
        discovery.run(discovery_registry).await;
    });

    let app = router(state);

    let address = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&address).await?;

    tracing::info!("starting node {} on {}", config.node_id, address);

    axum::serve(listener, app).await?;
    
    Ok(())
}
