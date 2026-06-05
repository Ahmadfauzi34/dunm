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

## 2024-05-28 - Corrected Tensor Identity Check Math
**Learning:** In the previous tensor identity optimization attempt, the condition used for circular convolution identity was flawed. The true mathematical identity for circular convolution in the time domain is the Kronecker delta (an array with `1.0` at index `0`, and `0.0` everywhere else), rather than a scalar multiplier or bounds-end `1.0`. Creating an incorrectly mathematically bounded shortcut check slows down the pipeline since the condition is never met, wasting $O(N)$ lookup time before executing the $O(N \log N)$ FFT transform.
**Action:** Replaced the flawed bounds check with a proper delta function check `(a[0] - 1.0).abs() <= EPSILON` and zeros for the rest using iterator methods. Fixed `TopDownAxiomator`'s internal identity tensor generation to match this true mathematical delta function. Using an early exit via `a[0] == 0.0` and `a.iter().skip(1).all(|v| v == 0.0)` enables lightning-fast $O(1)$ exits for default/zero cases without wasting time scanning the full array.
## 2026-05-18 - Perbaikan MCTS Depth Check & Sanitasi PR
**Learning:** `CognitivePhase::MacroStructural` di `quantum_search.rs` sebelumnya salah dievaluasi pada `current_depth <= 1`, menyebabkan evaluasi yang seharusnya untuk piksel mikroskopis dilewati. Hal ini menyebabkan error pragmatis negatif tinggi diartikan sebagai *ground state*. Selain itu, meninggalkan script scratchpad Python `.py` di indeks *working tree* merusak standar kebersihan repositori.
**Action:** Evaluasi MacroStructural dipaksa *strictly* untuk `current_depth == 0`. Selalu bersihkan *working tree* dari skrip scratchpad atau `.log` sebelum meng-commit dan merilis patch, menggunakan `git rm --cached` atau menambahkannya ke `.gitignore`.
## 2026-05-18 - .gitignore override
**Learning:** Ketika menambahkan script `.py` penting (seperti utility tools) sementara repositori memiliki `.gitignore` global untuk `*.py`, file tersebut tidak akan ter-add dengan `git add` biasa.
**Action:** Gunakan flag `-f` (`git add -f path/to/script.py`) untuk memaksa memasukkan utility tool tersebut tanpa harus mengubah `.gitignore` global (sehingga tidak merusak isolasi scratchpad).

## 2026-05-18 - Atomic / Sparse Skills for Recursive MCTS
**Learning:** Menggabungkan banyak tindakan spasial/fisika ke dalam satu aksioma kaku (contoh: mengekstrapolasi semua 4 sudut sekaligus) akan membanjiri manifold dengan *noise*. Hal ini memicu penalti *Pragmatic Error* yang parah pada MCTS, yang berakibat pada pemangkasan (pruning) cabang yang seharusnya memiliki solusi benar (amplitudo memudar).
**Action:** Saat mengembangkan atau memperluas pipa logika (`TopDownAxiomator` atau `skill_ontology`), pastikan kemampuan/skill dirancang secara **Sparse dan Atomic** (misal: pecah menjadi varian `TL_BR` dan `TR_BL`). Ini memungkinkan agen MCTS untuk menggunakan *skill* secara rekursif dan mandiri, serta membuang percabangan yang tidak perlu tanpa mengorbankan tebakan yang benar.

## 2024-06-05 - [HierarchicalPlanner Zero-Grid Validation Fix]
**Learning:** Evaluasi simulasi MCTS pada tahap validasi (`ValidationCheck::ExactMatch`) menggunakan representasi matriks nol statis (`dummy_grid`) menyebabkan seluruh state valid gagal secara komprehensif, mengacaukan perhitungan `pragmatic_error` oleh `SimdEnergyCalculator` yang seharusnya mengevaluasi tingkat akurasi piksel secara mikroskopis.
**Action:** Alih-alih menggunakan matriks yang berisi nol, kita mendelegasikan tugas ke `HologramDecoder::collapse_to_grid` untuk merender/mentransformasi `EntityManifold` state aktual (`expected`) ke dalam bentuk spasial matriks 2D sebelum menghitung `pragmatic_error`. Hal ini menjamin perbandingan 1-1 akurat dengan representasi tensor state dalam lingkungan *multiverse*.
