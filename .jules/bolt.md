## 2024-05-28 - Fast Cosine Similarity
**Learning:** `Array1::iter().zip().map().sum()` and mapping `mag_a` separately is inefficient due to multiple iterator passes and overhead. A single manual loop accumulating the dot product and both squared magnitudes reduces calculation time by ~66% (from 378ms to 126ms for 10k similarity calculations on dim=8192).
**Action:** Implemented a new, optimized version of `FHRR::similarity` which is a critical hotspot used frequently throughout the reasoning loop for similarity comparisons.

## 2024-05-16 - Topological incidence matrices optimization
**Learning:** In topological incidence matrices (like edge-to-triangle boundary matrices), lower-dimensional simplices (e.g., edges) constructed via ordered nested loops are naturally sorted. We can dramatically improve boundary operator construction from O(Triangles * Edges) to O(Triangles * log(Edges)) by using binary search instead of nested loops. The performance baseline improved from ~104.4ms to ~76.7ms for n=500.
**Action:** When constructing hierarchical topological structures, use `binary_search` for naturally sorted elements and explicitly encode the structural invariants using `debug_assert!(edges.windows(2).all(|w| w[0] < w[1]));` to verify correctness without impacting release performance.
