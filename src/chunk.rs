use std::ptr::NonNull;

pub struct Chunk {
    base: NonNull<u8>, // 对齐后的起始地址
    capacity: usize,   // 字节数
    // drop 语义：若放 T 且 T: Drop，需要记录每个 slot 是否已初始化
    init_flags: Option<Vec<_>>, // 或者干脆要求 T 不实现 Drop/简单处理
}
