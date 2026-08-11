use std::ptr;

use crate::chunk::Chunk;
use crate::freelist::FreeList;
use crate::util::{self, SIZE_CLASSES};

const CHUNK_BYTES: usize = 16 * 1024;

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
    slot_size: usize,
    slots_per_chunk: usize,
    chunks: Vec<Chunk>,
    free: FreeList,
}

impl ClassPool {
    fn new(slot_size: usize) -> Self {
        Self {
            slot_size,
            slots_per_chunk: CHUNK_BYTES / slot_size,
            chunks: Vec::new(),
            free: FreeList::new(),
        }
    }

    fn alloc(&mut self) -> *mut u8 {
        let slot = match self.free.pop() {
            Some(slot) => slot,
            None => {
                self.grow();
                self.free
                    .pop()
                    .expect("FreeList should not be empty after grow")
            }
        };
        self.slot_addr(slot) as *mut u8
    }

    fn dealloc(&mut self, addr: usize, size: usize) -> bool {
        let (chunk_idx, slot_in_chunk) = match self.find_slot(addr, size) {
            Some(pos) => pos,
            None => return false,
        };
        let slot = chunk_idx * self.slots_per_chunk + slot_in_chunk;
        self.free.push(slot);
        true
    }

    fn grow(&mut self) {
        self.chunks.push(Chunk::new(CHUNK_BYTES, self.slot_size));
        self.free.push_range(self.slots_per_chunk);
    }

    fn slot_addr(&self, slot: usize) -> usize {
        let (chunk_idx, slot_in_chunk) = (slot / self.slots_per_chunk, slot % self.slots_per_chunk);
        self.chunks[chunk_idx].slot_addr(slot_in_chunk)
    }

    fn find_slot(&self, addr: usize, size: usize) -> Option<(usize, usize)> {
        if size > self.slot_size {
            return None;
        }
        for (chunk_idx, chunk) in self.chunks.iter().enumerate() {
            let start = chunk.base();
            let end = start + chunk.capacity();
            if addr >= start && addr.checked_add(size).is_some_and(|e| e <= end) {
                let offset = addr - start;
                // 必须在槽边界上（整数个 slot_size 偏移）
                if !offset.is_multiple_of(self.slot_size) {
                    return None;
                }
                return Some((chunk_idx, offset / self.slot_size));
            }
        }
        None
    }
}

impl Default for Pool {
    fn default() -> Self {
        Self::new()
    }
}

impl Pool {
    pub fn new() -> Self {
        let classes = SIZE_CLASSES.map(ClassPool::new).into();
        Self {
            classes,
            stats: PoolStats::default(),
        }
    }

    // 按字节分配。size 超过最大档或 align 超过槽大小时拒绝，返回 null
    pub fn alloc(&mut self, size: usize, align: usize) -> *mut u8 {
        if size == 0 || size > util::max_class_size() {
            return ptr::null_mut();
        }
        let class = &mut self.classes[util::size_class_index(size)];
        if align > class.slot_size {
            return ptr::null_mut();
        }

        let addr = class.alloc();
        if addr.is_null() {
            return addr;
        }
        self.stats.alloc_count += 1;
        self.stats.current_live += 1;
        self.stats.peak_live = self.stats.peak_live.max(self.stats.current_live);
        addr
    }

    /// 按字节归还。ptr 必须是本池 alloc 的返回值，size 必须与 alloc 时一致；
    /// 非法 ptr/size 会被静默忽略（阶段 A 不做更严格的失败语义）
    ///
    /// # Safety
    ///
    /// - `ptr` 必须来自本池的 `alloc`，且 size/align 与原 alloc 一致
    /// - `ptr` 在调用期间必须未被归还
    pub unsafe fn dealloc(&mut self, ptr: *mut u8, size: usize, _align: usize) {
        if ptr.is_null() || size == 0 {
            return;
        }
        let class = &mut self.classes[util::size_class_index(size)];
        if !class.dealloc(ptr as usize, size) {
            return;
        }
        self.stats.free_count += 1;
        self.stats.current_live = self.stats.current_live.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocations_are_aligned_and_distinct() {
        let mut pool = Pool::new();
        let mut ptrs = Vec::new();
        for _ in 0..256 {
            let p = pool.alloc(32, 8);
            assert!(!p.is_null());
            assert_eq!(p as usize % 8, 0);
            assert!(!ptrs.contains(&(p as usize)));
            ptrs.push(p as usize);
        }
        assert_eq!(pool.stats.current_live, 256);
    }

    #[test]
    fn freed_memory_is_reused() {
        let mut pool = Pool::new();
        let first = (0..64)
            .map(|_| pool.alloc(64, 8) as usize)
            .collect::<Vec<_>>();
        for &p in &first {
            unsafe { pool.dealloc(p as *mut u8, 64, 8) };
        }

        let second = (0..64)
            .map(|_| pool.alloc(64, 8) as usize)
            .collect::<Vec<_>>();
        let (mut f, mut s) = (first.clone(), second.clone());
        f.sort_unstable();
        s.sort_unstable();
        assert_eq!(f, s); // 全部归还的格子都被重新拿到
    }

    #[test]
    fn data_integrity_roundtrip() {
        let mut pool = Pool::new();
        let p = pool.alloc(8, 8) as *mut u64;
        assert!(!p.is_null());
        unsafe {
            p.write(0xDEAD_BEEF_CAFE_F00D);
            assert_eq!(p.read(), 0xDEAD_BEEF_CAFE_F00D);
        }
    }

    #[test]
    fn oversized_or_misaligned_requests_are_rejected() {
        let mut pool = Pool::new();
        assert!(pool.alloc(0, 8).is_null());
        assert!(pool.alloc(util::max_class_size() + 1, 8).is_null()); // 超最大档
        assert!(pool.alloc(16, 32).is_null()); // align > slot_size
    }

    #[test]
    fn stats_track_live_allocations() {
        let mut pool = Pool::new();
        let a = pool.alloc(16, 8);
        let b = pool.alloc(16, 8);
        assert_eq!(pool.stats.alloc_count, 2);
        assert_eq!(pool.stats.current_live, 2);
        assert_eq!(pool.stats.peak_live, 2);

        unsafe { pool.dealloc(a, 16, 8) };
        assert_eq!(pool.stats.free_count, 1);
        assert_eq!(pool.stats.current_live, 1);
        assert_eq!(pool.stats.peak_live, 2);

        unsafe { pool.dealloc(b, 16, 8) };
        assert_eq!(pool.stats.free_count, 2);
        assert_eq!(pool.stats.current_live, 0);
    }

    #[test]
    fn dealloc_with_wrong_size_is_ignored() {
        let mut pool = Pool::new();
        let p = pool.alloc(16, 8);
        unsafe { pool.dealloc(p, 128, 8) }; // 按 128B 归还 -> 找错 class
        assert_eq!(pool.stats.free_count, 0);
        assert_eq!(pool.stats.current_live, 1);

        unsafe { pool.dealloc(p, 16, 8) }; // 用正确 size 仍可正常归还
        assert_eq!(pool.stats.free_count, 1);
        assert_eq!(pool.stats.current_live, 0);
    }
}
