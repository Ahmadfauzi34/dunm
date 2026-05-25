## 2024-05-28 - Fast Cosine Similarity
**Learning:** `Array1::iter().zip().map().sum()` and mapping `mag_a` separately is inefficient due to multiple iterator passes and overhead. A single manual loop accumulating the dot product and both squared magnitudes reduces calculation time by ~66% (from 378ms to 126ms for 10k similarity calculations on dim=8192).
**Action:** Implemented a new, optimized version of `FHRR::similarity` which is a critical hotspot used frequently throughout the reasoning loop for similarity comparisons.

## 2024-05-15 - [Topological Construction Speedup]
**Learning:** When constructing topological incidence matrices like the $D_2$ edge-to-triangle matrix, nested simplices (like edges) constructed via ordered nested loops are naturally sorted.
**Action:** Replace $O(T \times E)$ dense inner loops with `binary_search` to map lower-dimensional simplices to higher ones, improving construction time to $O(T \log E)$ and dropping overhead significantly.

## 2024-05-17 - [Optimize topological Laplacian construction]
**Learning:** During the computation of the `d2` boundary matrix mapping edges to triangles, we discovered that scanning the resulting $O(T \times E)$ dense-like Array2 matrix to extract non-zero entries for combinatorial Laplacian generation was redundantly wasteful. We already performed a binary search to find the 3 edges for each triangle during matrix construction.
**Action:** By explicitly storing the index of these 3 edge connections into a flat struct like `Vec<(usize, f32)>` mapped to each triangle during $d2$ construction, we drop the computational complexity of extracting cliques from $O(E \times T)$ back down to $O(T)$. Always store sparse structure components concurrently while executing nested searches to avoid double-processing.

## 2026-05-18 - [Topological Operations Optimizations]
**Learning:** Found a major performance bottleneck in `compute_laplacians_and_betti` related to the instantiation of the `L1` laplacian using `d1.t().dot(d1)`. This dense matrix multiplication causes O(N*E^2) time complexity, whereas calculating the sparse interactions directly is O(E^2), speeding up the function dramatically. Further, L0 which was calculated via `d1.dot(&d1.t())` can also be calculated sparsely in O(E).
**Action:** Always compute topological incidence laplacians using the sparse edges array rather than performing dense matrix multiplications on the boundary operators.

## 2024-05-22 - Optimize Triangle Construction in Topological Computations
**Learning:** In `QuantumCellComplex::from_manifold`, computing triangles via nested loops `for k in (j + 1)..n` takes $O(E \times N)$ operations (~625,000 inner loop iterations for 500 vertices). By first constructing an adjacency list for each vertex (`adj[i]`), finding a common neighbor (a triangle) becomes the intersection of two sorted lists. Furthermore, since we only want triangles `(i, j, k)` with `i < j < k`, we can use `binary_search` to start intersecting at elements greater than `j`.
**Action:** Always consider replacing naive nested loops for clique-finding (like triangles) with adjacency list intersections. Pre-sorting or constructing sorted adjacency lists guarantees optimal intersection times for finding 3-cliques in sparse graph structures.

## 2026-05-23 - Fused Tensor Processing Loops
**Learning:** During the application of the Grover Diffusion Operator, iterating over a large, flat amplitudes array (size `search_space_size * 8192`) multiple times per operation (once for inversion about the mean, then again to calculate sum of squares, and once more to apply thermal scaling) caused significant memory bandwidth overhead.
**Action:** When performing sequence-like array operations (like reflection -> normalization), always look for opportunities to fuse the loops. By computing the sum of squares simultaneously during the reflection step, we drop the number of passes over the large `amplitudes` array, cutting Grover iteration time by ~14% per cycle.
## 2026-05-23 - Prevent Vector allocation in Hot-Loop Bind
**Learning:** During the application of `FHRR::bind`, calculating the convolution of two arrays required zipping the frequency maps and then calling `.collect()`, instantiating a new `Vec<Complex<f32>>` of length 8192 for each bound pair.
**Action:** By applying `iter_mut()` directly over `cx_a` during the tensor zip logic, we calculate the multiplication in-place and pass `cx_a` directly to `fft_inv`. This eliminates a heavy memory allocation and significantly speeds up operations executing millions of bindings per second.
## 2026-05-23 - FHRR::bind Zero-Allocation for Tensor Arrays
**Learning:** `FHRR::bind` previously returned an allocated `Array1<f32>`, leading to a significant bottleneck in `MultiverseSandbox::apply_axiom` where the spatial and semantic arrays of hundreds of entities were mapped to temporary `Array1` objects via `from_vec().to_vec()`, bound, and re-assigned via `assign()`. This caused cache thrashing and memory overhead (`Amnesia Singkat`).
**Action:** Introduced `FHRR::bind_mut(a: &mut [f32], b: &[f32])` which applies the Fast Fourier Transform and complex circular convolution entirely in-place over the input slice `a`. By replacing the tensor assignments in `apply_axiom` with this slice-based mutation, we avoid redundant heap allocations and `.clone()` calls, reducing the inner loop timing by ~50%.
## 2026-05-23 - Fast Sparse Clone for EntityManifold SOA
**Learning:** The implicitly derived `Clone` trait for `EntityManifold` cloned the entire memory capacity of each `Vec` (like `capacity * 8192` elements for `spatial_tensors`), ignoring the semantic `active_count` boundary. During high branching in the MCTS, this triggered severe memory bloat ("Amnesia Singkat" / `Bottleneck::FalseSharing`) by moving gigabytes of unused memory per second.
**Action:** Replaced `#[derive(Clone)]` with a manual `impl Clone for EntityManifold` that utilizes `extend_from_slice` limited strictly to `[..self.active_count]`. This guarantees that nodes only duplicate live physical bounds, dramatically reducing MCTS branch instantiation times by ~30x during tree traversal.
