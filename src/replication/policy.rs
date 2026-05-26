#[derive(Debug, Clone)]
pub struct ReplicationPolicy {
    pub replication_factor: usize,
}

impl ReplicationPolicy {
    pub fn new(replication_factor: usize) -> Self {
        Self {
            replication_factor: replication_factor.max(1),
        }
    }

    pub fn remote_replica_count(&self) -> usize {
        self.replication_factor.saturating_sub(1)
    }
}
