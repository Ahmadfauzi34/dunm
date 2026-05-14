## 2024-05-28 - Fast Cosine Similarity
**Learning:** `Array1::iter().zip().map().sum()` and mapping `mag_a` separately is inefficient due to multiple iterator passes and overhead. A single manual loop accumulating the dot product and both squared magnitudes reduces calculation time by ~66% (from 378ms to 126ms for 10k similarity calculations on dim=8192).
**Action:** Implemented a new, optimized version of `FHRR::similarity` which is a critical hotspot used frequently throughout the reasoning loop for similarity comparisons.
## 2024-05-14 - Optimize d2 boundary matrix creation in Quantum Topology
**Learning:** In `QuantumCellComplex::from_manifold`, the `edges` list is naturally sorted because it is constructed using nested loops `for i in 0..n` and `for j in (i+1)..n`. This means we can replace $O(E)$ linear scans with $O(\log E)$ binary searches when mapping edges to triangles.
**Action:** When constructing topological incidence matrices (like edge-to-triangle matrices), check if the lower-dimensional simplices are already sorted. If so, use `binary_search` instead of nested loops. This reduces the time to build `d2` from $O(T \times E)$ to $O(T \log E)$.
