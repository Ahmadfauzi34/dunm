## 2024-05-28 - Fast Cosine Similarity
**Learning:** `Array1::iter().zip().map().sum()` and mapping `mag_a` separately is inefficient due to multiple iterator passes and overhead. A single manual loop accumulating the dot product and both squared magnitudes reduces calculation time by ~66% (from 378ms to 126ms for 10k similarity calculations on dim=8192).
**Action:** Implemented a new, optimized version of `FHRR::similarity` which is a critical hotspot used frequently throughout the reasoning loop for similarity comparisons.

## 2024-05-15 - [Topological Construction Speedup]
**Learning:** When constructing topological incidence matrices like the $D_2$ edge-to-triangle matrix, nested simplices (like edges) constructed via ordered nested loops are naturally sorted.
**Action:** Replace $O(T \times E)$ dense inner loops with `binary_search` to map lower-dimensional simplices to higher ones, improving construction time to $O(T \log E)$ and dropping overhead significantly.

## 2024-05-17 - [Optimize topological Laplacian construction]
**Learning:** During the computation of the `d2` boundary matrix mapping edges to triangles, we discovered that scanning the resulting $O(T \times E)$ dense-like Array2 matrix to extract non-zero entries for combinatorial Laplacian generation was redundantly wasteful. We already performed a binary search to find the 3 edges for each triangle during matrix construction.
**Action:** By explicitly storing the index of these 3 edge connections into a flat struct like `Vec<(usize, f32)>` mapped to each triangle during $d2$ construction, we drop the computational complexity of extracting cliques from $O(E \times T)$ back down to $O(T)$. Always store sparse structure components concurrently while executing nested searches to avoid double-processing.
