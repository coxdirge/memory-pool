# memory-pool

**A memory pool demo implemented in Rust.**

一个从零实现的内存池（Memory Pool / Slab Allocator）学习项目，结构参考 TCMalloc 的分层设计。当前完成 **阶段 A（单线程定长块池 + size class）**，阶段 B/C（ThreadCache / CentralCache / PageCache）为后续目标。

An educational memory-pool (slab allocator) project built from scratch, following a TCMalloc-style layered design. Currently at **Stage A: single-threaded fixed-size block pool + size classes**; Stages B/C (ThreadCache / CentralCache / PageCache) are planned.

## 特性 / Features

- **10 档 size class**：`8, 16, 32, 64, 128, 256, 512, 1024, 2048, 8192` 字节，请求自动向上取整到第一个能装下的档位（`partition_point` 二分查找）。
- **16KB Chunk**：每次扩容从系统按 16KB 申请一大块，按槽大小切成等大格子，槽边界天然对齐。
- **索引式 LIFO 空闲链表**：用 `head + next: Vec<usize>` 记录空闲槽，非侵入式——不在被释放的内存里写指针。
- **零第三方依赖**：纯 `std::alloc`，Rust edition 2024。
- **内置统计**：`alloc_count / free_count / current_live / peak_live`，随时确认池是否泄漏。

- **10 size classes**: `8, 16, 32, 64, 128, 256, 512, 1024, 2048, 8192` bytes; requests round up to the smallest class that fits (binary search via `partition_point`).
- **16KB chunks**: each growth asks the OS for one 16KB block, sliced into equal slots whose boundaries are naturally aligned.
- **Index-based LIFO free list**: `head + next: Vec<usize>` — non-intrusive, never writes pointers into freed memory.
- **Zero dependencies**: pure `std::alloc`, Rust edition 2024.
- **Built-in stats**: `alloc_count / free_count / current_live / peak_live` to check for leaks.

## 快速开始 / Quick Start

```bash
cargo build    # 构建 / build
cargo run      # 运行演示程序 / run the demo
cargo test     # 运行 11 个单元测试 / run the 11 unit tests
```

`cargo run` 的演示流程：分配 4 个 8 字节槽 → 写入 → 全部归还 → 再分配 1 个（验证复用）→ 打印统计。输出示例（地址每次运行不同）：

```text
alloc #0: 0xb9ac00000
alloc #1: 0xb9ac00008
alloc #2: 0xb9ac00010
alloc #3: 0xb9ac00018
dealloc done
realloc: 0xb9ac00018    <- should be the same as the first alloc
stats: PoolStats { alloc_count: 5, free_count: 5, current_live: 0, peak_live: 4 }
```

## 架构 / Architecture

```
memory-pool/
├── Cargo.toml            # 零依赖
├── src/
│   ├── lib.rs            # 模块组织
│   ├── main.rs           # 演示入口（分配/归还/复用/统计）
│   ├── pool.rs           # Pool 主 API + PoolStats（核心）
│   ├── chunk.rs          # Chunk：16KB 连续块切成等大槽
│   ├── freelist.rs       # FreeList：索引式 LIFO 空闲链表
│   ├── util.rs           # SIZE_CLASSES / size_class_index / round_up
│   ├── threadcache.rs    # 【占位】阶段 B：ThreadCache（TLS）
│   ├── central.rs        # 【占位】阶段 C：CentralCache（桶锁）
│   ├── page.rs           # 【占位】阶段 C：PageCache（页 span）
│   └── bench/            # 【预留】criterion 基准
```

**分配流程 / Allocation path**：

```
alloc(size, align)
  → 拒绝 size==0 / size > 8192 / align > slot_size
  → size_class_index(size) 定位档位 ClassPool
  → FreeList.pop() 弹空闲槽索引
  → 空闲列表为空则 grow()：申请新 16KB Chunk，push_range 全部槽索引
  → 返回槽地址，更新统计
```

**释放流程 / Deallocation path**：`dealloc(ptr, size, align)` → 按 size 定位档位 → 槽索引压回 FreeList（LIFO）。

## 核心 API / Public API

| 方法 / Method | 签名 / Signature | 说明 / Notes |
| --- | --- | --- |
| `Pool::new` | `() -> Pool` | 创建池（也实现 `Default`） |
| `Pool::alloc` | `(&mut self, size: usize, align: usize) -> *mut u8` | 拒绝 `size == 0`、`size > 8192`、`align > slot_size`，失败返回 `null` |
| `Pool::dealloc` | `unsafe fn (&mut self, ptr: *mut u8, size: usize, align: usize)` | `size` 必须与 `alloc` 时一致；非法参数静默忽略 |
| `Pool::stats` | `pub PoolStats` | `alloc_count / free_count / current_live / peak_live` |

## 测试 / Tests

11 个内联单元测试（`#[cfg(test)]`，无 `tests/` 目录）：

| 模块 | 数量 | 覆盖内容 |
| --- | --- | --- |
| `pool.rs` | 6 | 分配/复用/统计等 |
| `freelist.rs` | 3 | pop / push / push_range |
| `util.rs` | 2 | size class 换算 |

## 路线图 / Roadmap

对齐设计文档 `docs/memory-pool/06-api-draft.md` 的 6.0 阶段划分：

| 阶段 | 目标 | 状态 |
| --- | --- | --- |
| **A** | 单线程定长块池 + size class | ✅ 已完成（当前） |
| **B** | ThreadCache（`thread_local!` TLS 无锁路径 + 批量借还） | 🔜 `src/threadcache.rs` 占位 |
| **C** | 完整三层：CentralCache（桶锁）+ PageCache（页 span） | 🔜 `src/central.rs` / `src/page.rs` 占位 |

`src/bench/` 预留用于阶段 C 之后的 criterion 性能基准。

## 配套学习文档 / Companion Docs

项目配套一份系统学习文档：**`docs/memory-pool/`**（Obsidian vault，中文），从"为什么需要内存池"讲到"三层并发架构"：

- `01-why-memory-pool.md` → 动机与背景
- `02-memory-fundamentals.md` → 内存布局、对齐、碎片
- `03-rust-unsafe-basics.md` → Rust 内存模型与 unsafe
- `04-allocator-apis.md` → GlobalAlloc / Allocator 接口
- `05-design-spectrum.md` → 设计谱系：Bump → Free List → Slab + size class → 三层并发（核心章节）
- `06-api-draft.md` → API 设计草案（本仓库的实现蓝图）
- `07-common-pitfalls.md` → UB 与陷阱清单
- `08-references.md` → 参考资料

## License

[MIT](LICENSE) © 2026 coxdirge
