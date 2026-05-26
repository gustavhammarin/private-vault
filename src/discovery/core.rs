use crate::{discovery::registry::PeerRegistry, models::peer::Peer};

#[derive(Debug, Clone)]
pub struct PeerDiscoveryService{
    peers_base_urls: Vec<String>,
    client: reqwest::Client,
}

impl PeerDiscoveryService {
    pub fn new(peers_base_urls: Vec<String>) -> Self {
        Self { 
            peers_base_urls, 
            client: reqwest::Client::new() 
        }
    }

    pub async fn discover_once(&self, registry: &PeerRegistry) {
        for url in &self.peers_base_urls {
            let response = match self.client.get(format!("{url}/peer")).send().await {
                Ok(response) => response,
                Err(err) => {
                    tracing::warn!("failed to reach peer {}: {:?}", url, err);
                    continue;
                }
            };
    
            if !response.status().is_success() {
                tracing::warn!("peer {} returned {}", url, response.status());
                continue;
            }
    
            match response.json::<Peer>().await {
                Ok(peer) => registry.upsert(peer).await,
                Err(err) => tracing::warn!("failed to parse peer {}: {:?}", url, err),
            }
        }
    }
    pub async fn run(self, registry: PeerRegistry) {
        loop {
            self.discover_once(&registry).await;
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}


