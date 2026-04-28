# 📜 BUKU HITAM ARSITEKTUR RRM - FRACTAL AI EDITION

**Karakteristik Utama Fraktal dalam Recursive Reasoning**

> **Versi:** 5.0-Fraktal
> **Domain:** Self-Similar AI, Infinite Detail Systems, Fractional Dimension Intelligence
> **Prinsip Utama:** "Bagian mencerminkan keseluruhan, keseluruhan terbentuk dari bagian"

---

## 🌀 1. FILOSOFI FRACTAL AI: Self-Similarity dalam Arsitektur

Setiap layer AI memiliki struktur **IDENTIK** dengan layer lainnya. Bedanya hanya pada: `scale`, `granularity`, dan `time-horizon`.

```rust
/// Struktur dasar Fraktal AI
pub struct FractalNode {
    /// Self: Identitas node ini (level, x, y, z, t)
    pub id: FractalId,

    /// Similarity: Struktur yang sama di semua level
    pub perception: PerceptionField,      // Lihat "apa yang ada"
    pub reasoning: ReasoningEngine,       // Pikir "apa artinya"
    pub action: ActionGenerator,          // Lakukan "apa yang perlu"
    pub memory: MemoryStore,              // Ingat "untuk masa depan"

    /// Recursive: Pointer ke parent dan children (sama struktur!)
    pub parent: Option<Box<FractalNode>>, // Level n+1 (coarser)
    pub children: Vec<FractalNode>,       // Level n-1 (finer)

    /// Scale: Parameter yang beda per level
    pub scale: FractalScale,
}

pub struct FractalScale {
    pub spatial_resolution: f32,   // 1.0 (coarse) → 0.001 (fine)
    pub temporal_horizon: f32,     // Years → Seconds
    pub abstraction_level: u8,     // 0 (concrete) → 10 (abstract)
    pub confidence_threshold: f32, // Lower = finer detail needed
}
```

**Self-Similarity Pattern:**
```text
Level 3 (Strategic):  "Selesaikan Task ARC"
├── Level 2 (Tactical): "Identifikasi Pola Geometris"
│   ├── Level 1 (Operational): "Lakukan Crop pada Frame"
│   │   ├── Level 0 (Execution): "Terapkan Translasi Vektor"
│   │   │   └── Level -1 (Micro): "Hitung Tensor Spatial FHRR"
```
Semua level memiliki siklus kognitif yang sama: `perception → reasoning → action → memory`.

---

## 🌀 2. Infinite Detail: Zoom Tanpa Batas

Detail muncul saat diperlukan (On-Demand), tidak di-*pre-allocate* secara paksa, sehingga sesuai dengan prinsip **Zero-GC**.

```rust
/// Detail muncul saat diperlukan, dievaluasi secara malas (lazy)
pub struct InfiniteDetailField {
    /// Coarse representation (selalu ada di memori utama)
    pub coarse: TensorField,

    /// Fine details (lazy evaluation, on-demand)
    pub detail_cache: LruCache<DetailKey, TensorField>,

    /// Generator: Bisa membuat detail lebih halus
    pub detail_generator: Box<dyn DetailGenerator>,

    /// Maximum depth untuk mencegah infinite recursion
    pub max_depth: u8,
}
```

**Infinite Detail dalam Praktek ARC:**
- **Coarse:** "Ada grid berisi kotak-kotak" (Macro Bounding Box)
- **Zoom 1:** "Kotak-kotak itu membentuk pola checkerboard" (Topology)
- **Zoom 2:** "Beberapa kotak memiliki noise warna" (Pixel-level)
- **Zoom 3:** "Noise warna tersebut sebenarnya adalah sinyal rotasi spasial" (Tensor FHRR level mikroskopis)

---

## 🌀 3. Dimensi Fraktal: Kompleksitas Berdimensi Pecahan

Dimensi menggambarkan "kompleksitas ruang". Dalam RRM, dimensi tidak harus integer (1D, 2D).

```rust
/// Kompleksitas dievaluasi berdasarkan "dimensi fraktal"
pub struct FractalDimension {
    pub topological_dim: u8,      // 1, 2, 3 (integer)
    pub fractal_dim: f32,         // 1.0 - 3.0 (fractional)
    pub measure: DimensionMeasure,
}

pub enum DimensionMeasure {
    Hausdorff { s: f32 },
    BoxCounting { epsilon: f32, n_boxes: usize },
    Information { entropy: f32 },
}
```

**Dimensi dalam Kognisi AI:**
- **Task Space 1.2D:** Linear task (Translasi sederhana).
- **Task Space 2.7D:** Kompleks (Dinamika multi-objek dengan nested rules).
- **Reasoning Space 1.8D:** *Branching logic* (MCTS) dengan backtracking.

---

## 🌀 4. Fraktal Alami & Adaptif

Fraktal alami seperti brokoli Romanesco: struktur yang sama, skala berbeda.

```rust
pub struct NaturalFractalAI {
    pub root: FractalGoal,
    pub branches: Vec<FractalGoal>,
    pub properties: NaturalProperties,
}

pub struct NaturalProperties {
    pub irregularity: f32,  // 0.0 = perfect fractal, 1.0 = random (noise kosmik)
    pub anisotropy: Vec3,   // Scale factors
    pub local_dimensions: HashMap<Region, FractalDimension>,
}
```

---

## 🌀 5. Fraktal Deterministik: Rigid Self-Similarity (Batas Mandelbrot)

Digunakan untuk membedakan antara perilaku stabil (deterministik) dan kreatif (chaotic/probabilistik).

```rust
pub struct DeterministicFractalAI {
    pub iteration_fn: Box<dyn Fn(Complex, Complex) -> Complex>,
    pub parameter: Complex,
    pub escape_radius: f32,
    pub max_iter: u32,
}
```
**Interpretasi (Cognitive Mode Shift):**
- **Inside Set:** Behavior stabil, predictable, reliable (`SimulationMode::StrictVSA`).
- **Outside Set:** Behavior chaotic, sensitive, creative (`SimulationMode::Probabilistic`).
- **Boundary:** *Edge of chaos*, paling adaptif, optimal untuk *Active Inference*.

---

## 🌀 6. InfiniteDetailField (Fractal Manifold / Muscle Layer)

Sebagai implementasi konkret dari prinsip *Infinite Detail*, entitas `EntityManifold` lama (yang mengalokasikan data mentah secara berlebihan) digantikan oleh `InfiniteDetailField`. Ini merupakan struktur **Copy-on-Write (CoW)** dengan hierarki Makro -> Meso -> Mikro yang diakses secara **On-Demand (Lazy Evaluation)**.

```rust
pub struct InfiniteDetailField {
    /// Level 0: Makro (selalu ada di memori, lightweight)
    pub coarse: MacroLevel,

    /// Level 1 & 2: Detail muncul on-demand (lazy), cached, bounded
    /// LRU eviction untuk OOM protection (menggunakan crate `lru`)
    pub detail_cache: RwLock<DetailCache>,

    /// Generator: Bisa membuat detail lebih halus dari coarse
    pub detail_generator: Arc<dyn DetailGenerator>,

    /// CoW State: Shared ownership antar level (Arc pointer ke data)
    pub shared_base: Arc<CoarseData>,
}

/// Zero-Copy Arc View (Opsi B: Aman dari Lifetime RwLock Guard)
pub enum ZeroCopyView {
    Macro {
        region_id: u32,
        data: Arc<CoarseData>, // Arc menjamin data tetap hidup tanpa memblokir cache
    },
    Meso {
        region_id: u32,
        local_idx: usize,
        data: Arc<MesoLevel>,  // Arc menjamin data tetap hidup tanpa memblokir cache
    },
    Micro {
        region_id: u32,
        local_idx: usize,
        data: Arc<MicroLevel>, // Arc menjamin data tetap hidup tanpa memblokir cache
    }
}
```

**Mekanisme Lazy Evaluation (OOM-Proof):**
- **MacroLevel:** Berisi bounding boxes kasar dan *dominant signatures*. Selalu *resident* di memori utama.
- **MesoLevel:** Mengandung *quadtree subdivisions*. Di-generate hanya jika *complexity* > 0.5 atau butuh zoom.
- **MicroLevel:** Mengandung detail *pixel-perfect* (1 piksel = 1 tensor Entity). Di-generate pada kondisi mendesak.
- **DetailCache:** LRU Cache dengan **budget_bytes hard limit**. Menghindari OOM pada MCTS dengan menggusur (evicting) detail-detail yang jarang diakses (kembali menjadi *Ghost States* di *parent*).

---

## 🎯 Kesimpulan: Arsitektur RRM V3 (The Fractal Vision)

> "Fraktal AI adalah RRM yang self-similar di semua scale, dengan detail yang muncul on-demand seperti zoom pada Mandelbrot set"

**Keuntungan Eksekusi:**
1. **Scale-Invariant:** Kode yang sama (`perceive -> reason -> act -> memory`) berlaku dari hierarki *planner* strategis hingga *tensor math* di tingkat mikroskopis.
2. **Adaptive Detail:** Efisien secara memori (Zero-GC compliant), detail di-*generate* hanya saat *confidence* rendah.
3. **Infinite Complexity:** Perilaku penyelesaian masalah yang *advanced* (emergent property) lahir dari pengulangan *rules* spasial yang sangat sederhana secara berulang-ulang di berbagai *scale*.

*"Seperti brokoli Romanesco: setiap bagian adalah brokoli, dan brokoli adalah setiap bagian"* 🥦🌀
