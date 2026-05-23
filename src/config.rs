use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub node_id: String,
    pub port: String,
    pub storage_path: String,
    pub peers: Vec<String>,
}

impl Config {
    pub fn load() -> Self {
        Self {
            node_id: env::var("NODE_ID").unwrap_or("unknown-node".to_string()),
            port: env::var("PORT").unwrap_or("8080".to_string()),
            storage_path: env::var("STORAGE_PATH").unwrap_or("./data".to_string()),
            peers: parse_peers(&env::var("PEERS").unwrap_or_default()),
        }
    }
}

fn parse_peers(value: &str) -> Vec<String> {
    value
        .split(",")
        .map(|peer| peer.trim().to_string())
        .filter(|peer| !peer.is_empty())
        .collect()
}
