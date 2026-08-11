use std::alloc::{Layout, alloc, dealloc};
use std::ptr::NonNull;

// 内存块，表示一段连续的内存区域，按照 slot_size 对齐，包含 capacity 个 slot
pub struct Chunk {
    base: NonNull<u8>, // 对齐后的起始地址
    capacity: usize,   // 字节数
    slot_size: usize,  // 每个 slot 的字节数
    #[allow(dead_code)]
    // 暂时不使用
    // drop 语义：若放 T 且 T: Drop，需要记录每个 slot 是否已初始化
    init_flags: Option<Vec<u8>>, // 每 bit 标记一个 slot 是否已初始化
}

impl Chunk {
    pub fn new(capacity: usize, slot_size: usize) -> Self {
        let layout =
            Layout::from_size_align(capacity, slot_size).expect("Invalid layout for Chunk");
        let base = unsafe { alloc(layout) };
        let base = NonNull::new(base).expect("allocate memory for Chunk");
        Self {
            base,
            capacity,
            slot_size,
            init_flags: None,
        }
    }

    #[inline]
    pub fn base(&self) -> usize {
        self.base.as_ptr() as usize
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub fn slot_addr(&self, i: usize) -> usize {
        self.base() + i * self.slot_size
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, self.slot_size).unwrap();
        unsafe { dealloc(self.base.as_ptr(), layout) };
    }
}
