## 2026-04-22 - [Performance/Memory Optimization] Removing Arc<Vec<T>> overhead in EntityManifold
**Learning:** `Arc::make_mut` inside a hot inner loop (like MCTS simulation) causes severe heap thrashing and deep copies when the strong count > 1. This defeats the purpose of Copy-on-Write for small arrays and destroys L1 cache locality, drastically slowing down simulations.
**Action:** Changed `EntityManifold` internal arrays to use plain `Vec<T>`. Relied on top-level contiguous `m.clone()` for states which is easily optimized by `memcpy` and eliminates locking and branching overhead for thousands of internal tensor mutations.


## 2026-05-10 - Domain-aware Matrix Operations vs Generic BLAS
**Learning:** In topology calculations, avoiding dense matrix multiplications (`O(N^3)`) by exploiting domain-specific topological invariants (e.g. triangles strictly bounded by 3 edges) creates massive performance gains. However, code reviewers often reject nested loops as "pessimizations" due to readability concerns and assumed BLAS superiority.
**Action:** Always present structural performance enhancements mathematically. Extract loops into named helpers describing the mathematical invariant (e.g., `add_triangle_clique_laplacian`) and avoid inline anonymous nested loops. Reuse vector allocations (`edges.clear()`) inside tight loops rather than re-allocating.
