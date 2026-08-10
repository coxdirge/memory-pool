pub struct FreeList {
    head: Option<usize>, // 当前空闲块索引
    next: Box<[usize]>,  // 下一空闲索引
}
