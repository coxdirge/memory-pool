use memory_pool::pool::Pool;

fn main() {
    let mut pool = Pool::new();

    let mut ptrs = Vec::new();
    for i in 0..4u64 {
        let p = pool.alloc(8, 8);
        assert!(!p.is_null());
        unsafe { (p as *mut u64).write(i) };
        println!("alloc #{i}: {p:p}");
        ptrs.push(p);
    }

    for &p in &ptrs {
        unsafe { pool.dealloc(p, 8, 8) };
    }
    println!("dealloc done");

    let p = pool.alloc(8, 8);
    println!("realloc: {p:p}    <- should be the same as the first alloc");
    unsafe { pool.dealloc(p, 8, 8) };

    println!("stats: {:?}", pool.stats);
}
