const SIZE_CLASSES: [usize; 10] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048, 8192];

// 输入请求大小 → 返回对应 class 的索引
fn size_class_index(size: usize) -> usize {
    // 可用于非二的幂次的索引
    // for (i, &class_size) in SIZE_CLASSES.iter().enumerate() {
    //     if size <= class_size {
    //         return i;
    //     }
    // }
    // SIZE_CLASSES.len() - 1

    if let Some(index) = next_power_of_two(size).trailing_zeros().checked_sub(3) {
        index as usize
    } else {
        SIZE_CLASSES.len() - 1
    }
}

// 输入请求大小 → 返回对齐后的实际块大小
fn round_up(size: usize) -> usize {
    if size <= SIZE_CLASSES[0] {
        SIZE_CLASSES[0]
    } else if size > SIZE_CLASSES[SIZE_CLASSES.len() - 1] {
        size
    } else {
        next_power_of_two(size)
    }
}

const fn next_power_of_two(mut size: usize) -> usize {
    if size == 0 {
        return 1;
    }
    size -= 1;
    size |= size >> 1;
    size |= size >> 2;
    size |= size >> 4;
    size |= size >> 8;
    size |= size >> 16;
    #[cfg(target_pointer_width = "64")]
    {
        size |= size >> 32;
    }
    size + 1
}

const fn prev_power_of_two(size: usize) -> usize {
    if size == 0 {
        return 0;
    }
    let lz = size.leading_zeros();
    1 << (usize::BITS - 1 - lz)
}
