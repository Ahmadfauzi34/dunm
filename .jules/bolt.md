## 2026-04-22 - [Performance/Memory Optimization] Removing Arc<Vec<T>> overhead in EntityManifold
**Learning:** `Arc::make_mut` inside a hot inner loop (like MCTS simulation) causes severe heap thrashing and deep copies when the strong count > 1. This defeats the purpose of Copy-on-Write for small arrays and destroys L1 cache locality, drastically slowing down simulations.
**Action:** Changed `EntityManifold` internal arrays to use plain `Vec<T>`. Relied on top-level contiguous `m.clone()` for states which is easily optimized by `memcpy` and eliminates locking and branching overhead for thousands of internal tensor mutations.

