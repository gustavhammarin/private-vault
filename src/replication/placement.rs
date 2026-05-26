use std::fmt::Debug;

use crate::replication::policy::ReplicationPolicy;

#[async_trait::async_trait]
pub trait ReplicaPlacement: Send + Sync + Debug {
    fn target_peers_for_file(
        &self,
        file_id: &str,
        peers: &[String],
        policy: &ReplicationPolicy,
    ) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct Blake3ReplicaPlacement;

#[async_trait::async_trait]
impl ReplicaPlacement for Blake3ReplicaPlacement {
    fn target_peers_for_file(
        &self,
        file_id: &str,
        peers: &[String],
        policy: &ReplicationPolicy,
    ) -> Vec<String> {
        let remote_count = policy.remote_replica_count();

        let mut scored_peers: Vec<([u8; 32], String)> = peers
            .iter()
            .map(|peer| {
                let score = placement_score(file_id, peer);
                (score, peer.clone())
            })
            .collect();
        
        scored_peers.sort_by_key(|(score, _)| *score);

        scored_peers
            .into_iter()
            .take(remote_count)
            .map(|(_, peer)| peer)
            .collect()
    }
}

fn placement_score(file_id: &str, peer: &str) -> [u8; 32] {
    let input = format!("file:{file_id}|peer:{peer}");
    *blake3::hash(input.as_bytes()).as_bytes()
}