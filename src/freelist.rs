const NIL: usize = usize::MAX;

pub struct FreeList {
    head: Option<usize>, // 当前空闲块索引
    next: Vec<usize>,    // 下一空闲索引
}

impl Default for FreeList {
    fn default() -> Self {
        Self::new()
    }
}

impl FreeList {
    pub fn new() -> Self {
        Self {
            head: None,
            next: Vec::new(),
        }
    }

    // 弹出一个空闲块索引
    pub fn pop(&mut self) -> Option<usize> {
        let i = self.head?;
        let next_i = self.next[i];
        self.head = if next_i == NIL { None } else { Some(next_i) };
        Some(i)
    }

    // 推入一个空闲块索引
    pub fn push(&mut self, i: usize) {
        self.next[i] = self.head.unwrap_or(NIL);
        self.head = Some(i);
    }

    // 推入一系列空闲块索引
    pub fn push_range(&mut self, count: usize) {
        let start = self.next.len();
        if count == 0 {
            return;
        }
        let old_head = self.head;
        for i in start..start + count {
            self.next.push(if i + 1 < start + count {
                i + 1
            } else {
                old_head.unwrap_or(NIL)
            });
        }
        self.head = Some(start);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pop_returns_slots_in_lifo_order() {
        let mut f = FreeList::new();
        f.push_range(4);
        assert_eq!(f.pop(), Some(0));
        assert_eq!(f.pop(), Some(1));
        assert_eq!(f.pop(), Some(2));
        assert_eq!(f.pop(), Some(3));
        assert_eq!(f.pop(), None);
    }

    #[test]
    fn freed_slot_is_reused() {
        let mut f = FreeList::new();
        f.push_range(4);
        assert_eq!(f.pop(), Some(0));
        f.push(0);
        assert_eq!(f.pop(), Some(0));
    }

    #[test]
    fn new_range_links_after_old_head() {
        let mut f = FreeList::new();
        f.push_range(2);
        assert_eq!(f.pop(), Some(0));
        assert_eq!(f.pop(), Some(1));
        assert_eq!(f.pop(), None);

        f.push_range(2); // slot 2、3 挂到链头
        assert_eq!(f.pop(), Some(2));
        assert_eq!(f.pop(), Some(3));
        assert_eq!(f.pop(), None);
    }
}
