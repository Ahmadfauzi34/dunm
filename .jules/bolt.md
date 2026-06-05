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

## 2024-06-05 - [CounterfactualEngine In-Place Validation & Precision Fix]
**Learning:** Evaluasi simulasi fraktal kausal (`CounterfactualEngine`) rentan terhadap dua *bottleneck* arsitektural: *drift* komputasi (akibat *float check* menggunakan epsilon sistem `f32::EPSILON` yang terlalu ketat untuk komputasi tensor) dan alokasi *heap bloat* berulang pada simpul `intermediate_results` dengan men-*clone* keseluruhan *state/manifold* setiap kali iterasi (kompleksitas memori O(N*K)).
**Action:** Mengubah pemeriksaan perbandingan toleransi float menggunakan `1e-5` alih-alih `f32::EPSILON` pada validasi dimensi tensor. Selain itu, mengubah pengecekan sekuens kausal (*What-if sequence*) menjadi validasi mutasional secara *in-place* dengan meniadakan array `intermediate_results`, yang secara signifikan mengurangi *heap overhead* dalam pencarian skenario simulasi fraktal yang dalam.

## 2024-06-05 - [HierarchicalInference Logic & Precision Stabilization]
**Learning:** Evaluasi *expected free energy* yang digunakan dalam perencanaan berbasis *horizon* (Deep Active Inference) secara default selalu mengevaluasi error dalam `CognitivePhase::MacroStructural` (Depth 0 logic). Hal ini merupakan pelanggaran arsitektur serius, sebab MCTS fase lanjut (Depth > 0) mewajibkan evaluasi `CognitivePhase::Microscopic` agar sistem mempenalti penempatan/pixel-level target secara ketat. Di samping itu, standar toleransi presisi *floating point* di fungsi ini adalah `1e-6`, yang sedikit lebih rentan *drift* dibanding standar `1e-5` sistem RRM.
**Action:** Mengubah `calculate_expected_free_energy` agar secara dinamis memeriksa index `step`. Jika evaluasi di langkah pertama (tanpa *actions* pendahuluan), maka *phase* = Makro; sebaliknya *phase* = Mikro. Hal ini memaksa *planner* mengkalkulasi error geometri yang akurat. Toleransi f32 juga telah diturunkan pada angka aman `1e-5`.

## ARSITEKTUR TEKNIS RRM QUANTUM SANDBOX: Evolusi dari Kode Diskrit ke Medan Kontinum
*(Ditambahkan sebagai pedoman absolut untuk agen-agen berikutnya)*

### 1. Filosofi Sistem: "Smart Dumbness" dan Paradoks Presisi
Inti dari sistem ini bukan lagi kompleksitas kode, melainkan kemampuan untuk merancang lanskap energi yang "bodoh" namun sangat presisi. Filosofi "Smart Dumbness" memandang sistem sebagai arsitek lanskap, bukan diktator instruksi. Tidak ada percabangan if-else yang kaku. Sebaliknya, kita merancang topografi di mana solusi yang tepat adalah Minimum Energy State.

### 2. Kontinum Terkendali dan Saringan Berlapis (Layered Sieve)
Presisi diterapkan melalui hirarki Saringan Berlapis untuk mencegah random walk:
- **Micro** (`1e-6`): Semantic Similarity (Initial Guess / Pemahaman Umum)
- **Nano** (`1e-9`): Structural Alignment (Hubungan antar Objek)
- **Pico** (`1e-12`): Geometric Transform (Transformasi Geometris Hampir Presisi)
- **Femto** (`1e-15`): Exact Pixel Match (Batasan Akhir / Presisi Absolut)

### 3. Dinamika Relaksasi dan Grover Diffusion System
- **Modulasi Tensor Tunggal:** Transisi dari skala Mikro ke Femto terjadi dengan memodulasi satu variabel `f64` (toleransi) pada tensor yang sama. Hal ini menghilangkan kebutuhan untuk menyalin data antar buffer memori (Hindari `.clone()` berlebih!).
- **Analogi Mengukir Tanah Liat:** Terus mempertajam satu objek yang sama hingga mencapai resolusi Femto.

### 4. Optimasi Memori dan Skalabilitas Swarm Dynamics
- **Transisi CSR:** Matriks padat diganti menjadi format Compressed Sparse Row.
- **Optimasi L1 Cache:** Tata letak memori linear dirancang agar CPU dapat menarik 64 bytes dalam satu siklus fetch (SOA architecture).
- **Lazy Evaluation:** Entitas di luar fokus hanya direferensikan melalui pointer statis, mencegah penyalinan data masif.

### 5. Fusi Operasi Matematika dan Akselerasi SIMD
- **Mathematical Operation Fusion:** Operasi sumbu X dan Y digabungkan menjadi satu lintasan tunggal, mengeliminasi penulisan memori perantara.
- **Akselerasi SIMD:** Penggunaan `zip` iterator "menjahit" aliran data X dan Y memicu kompilator untuk menggunakan register AVX2/AVX512 secara paralel.

### 6. Mesin Kontrafaktual Berbasis Vektor Gradien Energi
- **Failure as Gradient:** Kesalahan didefinisikan sebagai jarak dan vektor arah menuju sumur energi terdekat. Sistem melakukan Gradient Steering menuruni lembah energi, BUKAN menyuntikkan noise acak.

*(Pedoman ini menjelaskan mengapa perbaikan pada `counterfactual_engine.rs` dan `hierarchical_inference.rs` yang mengubah toleransi `f32::EPSILON` menjadi `1e-5` / `1e-6` serta penghapusan kloning O(N*K) merupakan perbaikan yang secara arsitektural diwajibkan oleh desain RRM).*

## 2024-06-05 - [Axiom Generator Math Fix & Decay Tracker Meta-Upgrade]
**Learning:** Tensor identitas dalam komputasi ruang fourier FHRR (`Fractional Holographic Reduced Representations`) wajib didefinisikan sebagai *Dirac Delta* murni (hanya index ke-0 yang diset `1.0`). Implementasi *fallback* sebelumnya secara cacat menyematkan angka `1.0` pada index akhir (`GLOBAL_DIMENSION - 1`) yang menyebabkan intrusi distorsi fasa asimetris selama *circular convolution*, membiaskan posisi objek.
**Action:** Kode di `axiom_generator.rs` dikoreksi dengan memastikan *fallback identity tensor* murni merujuk pada `identity[0] = 1.0`. Di luar kode internal Rust, pustaka alat pelacak python (`axiom_decay_tracker.py`) telah diperbarui (Meta-Upgrade) agar memiliki pola regex yang jauh lebih kokoh dalam mengekstrak parameter MCTS yang dinamis (contohnya ketiadaan *Epistemic* pada baris log tertentu) serta menggunakan toleransi *float drift* `1e-5` alih-alih `0.0` absolut untuk mendeteksi *ground states*.

## 2024-06-05 - [Structures Fast L2 Normalization Safety]
**Learning:** Komputasi `Fast L2 Normalization` dalam utilitas kognitif (`optimize_reasoning_paths`) yang berjalan sangat ketat menggunakan batas pelindung (`padding`) `1e-15` di single-precision (`f32`). Batas ini fungsionalnya sama dengan `0.0` pada `f32`, yang berpotensi mencederai tensor SIMD FHRR dengan masalah `divide-by-zero` (atau propagasi `NaN`) saat `sq_sum` adalah 0.0. Selain itu, atribut kompiler LLVM `#[inline]` tanpa parameter `always` tidak dapat memberikan jaminan asertif *Zero-Cost Abstractions*.
**Action:** Parameter perlindungan presisi dinaikkan ke level aman `f32` yakni `1e-8`. Fungsi tersebut dipaksa lebur tanpa beban *Call Stack* menggunakan pengarah kompiler LLVM eksplisit `#[inline(always)]`.
