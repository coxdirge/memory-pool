// 固定 2 的幂档位
pub const SIZE_CLASSES: [usize; 10] = [8, 16, 32, 64, 128, 256, 512, 1024, 2048, 8192];

// 输入请求大小 → 返回对应 class 的索引
pub fn size_class_index(size: usize) -> usize {
    // 可用于非二的幂次的索引
    // for (i, &class_size) in SIZE_CLASSES.iter().enumerate() {
    //     if size <= class_size {
    //         return i;
    //     }
    // }
    // SIZE_CLASSES.len() - 1

    // if let Some(index) = next_power_of_two(size).trailing_zeros().checked_sub(3) {
    //     index as usize
    // } else {
    //     SIZE_CLASSES.len() - 1
    // }

    SIZE_CLASSES
        .partition_point(|&class_size| class_size < size)
        .min(SIZE_CLASSES.len() - 1)
}

#[allow(dead_code)]
// 输入请求大小 → 返回对齐后的实际块大小
fn round_up(size: usize) -> usize {
    // if size <= SIZE_CLASSES[0] {
    //     SIZE_CLASSES[0]
    // } else if size > SIZE_CLASSES[SIZE_CLASSES.len() - 1] {
    //     size
    // } else {
    //     next_power_of_two(size)
    // }

    SIZE_CLASSES[size_class_index(size)]
}

pub fn max_class_size() -> usize {
    SIZE_CLASSES[SIZE_CLASSES.len() - 1]
}

// const fn next_power_of_two(mut size: usize) -> usize {
//     if size == 0 {
//         return 1;
//     }
//     size -= 1;
//     size |= size >> 1;
//     size |= size >> 2;
//     size |= size >> 4;
//     size |= size >> 8;
//     size |= size >> 16;
//     #[cfg(target_pointer_width = "64")]
//     {
//         size |= size >> 32;
//     }
//     size + 1
// }

// const fn prev_power_of_two(size: usize) -> usize {
//     if size == 0 {
//         return 0;
//     }
//     let lz = size.leading_zeros();
//     1 << (usize::BITS - 1 - lz)
// }

#[cfg(test)]
mod tests {
    use super::*;
    fn size_class_maps_to_first_fitting_slot() {
        assert_eq!(size_class_index(0), 0);
        assert_eq!(size_class_index(1), 0);
        assert_eq!(size_class_index(8), 0);
        assert_eq!(size_class_index(9), 1);
        assert_eq!(size_class_index(16), 1);
        assert_eq!(size_class_index(17), 2);
        assert_eq!(size_class_index(32), 2);
        assert_eq!(size_class_index(33), 3);
        assert_eq!(size_class_index(100), 4);
        assert_eq!(size_class_index(128), 4);
        assert_eq!(size_class_index(8192), 9);
        assert_eq!(size_class_index(100_000), 9);
    }

    #[test]
    fn round_up_returns_slot_size() {
        assert_eq!(round_up(1), 8);
        assert_eq!(round_up(20), 32);
        assert_eq!(round_up(4097), 8192);
    }
}
