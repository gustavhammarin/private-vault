use crate::models::ChunkMetadata;

pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub index: i64,
    pub hash: String,
    pub size: i64,
    pub data: Vec<u8>,
}

pub fn chunk_bytes(data: &[u8], chunk_size: usize) -> Vec<Chunk> {
    data.chunks(chunk_size)
        .enumerate()
        .map(|(index, chunk)| {
            let hash = blake3::hash(chunk).to_hex().to_string();
            Chunk {
                index: index as i64,
                hash,
                size: chunk.len() as i64,
                data: chunk.to_vec(),
            }
        })
        .collect()
}

pub fn to_metadata(chunks: &[Chunk]) -> Vec<ChunkMetadata>{
    chunks
        .iter()
        .map(|chunk| ChunkMetadata {
            index: chunk.index,
            hash: chunk.hash.clone(),
            size: chunk.size
        }).collect()
}
