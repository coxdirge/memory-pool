use std::cell::RefCell;

thread_local! {
    static TL_CACHE: RefCell<ThreadCache> = RefCell::new(ThreadCache::default());
}

struct ThreadCache {
    free_lists: [FreeList; N], // 每 size class 一条，全无锁
                               // 本地分配次数等统计
}
