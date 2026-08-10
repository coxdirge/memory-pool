use crate::chunk::Chunk;
use crate::freelist::FreeList;

pub struct Pool {
    classes: Box<[ClassPool]>,
    pub stats: PoolStats,
}

struct ClassPool {
    chunks: Vec<Chunk>,
    free: FreeList,
}
