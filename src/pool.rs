use crate::chunk::Chunk;
use crate::freelist::FreeList;

#[derive(Default, Debug, PartialEq, Eq)]
pub struct PoolStats {
    pub alloc_count: usize,
    pub free_count: usize,
    pub current_live: usize,
    pub peak_live: usize,
}

pub struct Pool {
    classes: Box<[ClassPool]>,
    pub stats: PoolStats,
}

struct ClassPool {
    chunks: Vec<Chunk>,
    free: FreeList,
}
