## 2024-05-28 - Fast Cosine Similarity
**Learning:** `Array1::iter().zip().map().sum()` and mapping `mag_a` separately is inefficient due to multiple iterator passes and overhead. A single manual loop accumulating the dot product and both squared magnitudes reduces calculation time by ~66% (from 378ms to 126ms for 10k similarity calculations on dim=8192).
**Action:** Implemented a new, optimized version of `FHRR::similarity` which is a critical hotspot used frequently throughout the reasoning loop for similarity comparisons.
