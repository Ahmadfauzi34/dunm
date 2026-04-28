# 🧬 RISET GENERATIF & MICRO-RELATION (RRM V3.1)

Melanjutkan fondasi **Fractal AI** (Makro-Meso-Mikro), riset ini menukik lebih dalam ke level **Mikro (Piksel)** dan **Generatif (Sintesis Skill)**.

Banyak task ARC tidak bisa diselesaikan dengan operasi makro kaku (seperti *crop*, *rotate*, atau translasi 1 objek besar). ARC menuntut agen untuk memahami **Micro-Relation** (hubungan piksel ke piksel, seperti "gambar garis dari titik merah ke titik biru") dan mampu **meng-generate skill baru** (*Generative Axioms*) di tengah penerbangan (runtime) jika *skill* dasar dari `SkillOntology` gagal.

---

## 🔬 1. MICRO-RELATION: Local Neighborhood Signatures

Di level *MicroLevel* (`InfiniteDetailField`), setiap entitas adalah piksel tunggal. Kita tidak lagi mengandalkan *bounding box*. Kita menggunakan **Quantum Entanglement (Resonansi FHRR)** untuk mendeteksi pola lokal.

### A. Konsep: Topological Field

Setiap piksel tidak hanya memiliki *spatial tensor* (X, Y) dan *semantic tensor* (Warna), tapi ia memancarkan **Local Neighborhood Signature (LNS)**. LNS adalah superposisi (penjumlahan) dari piksel-piksel di sekitarnya, dilemahkan oleh jarak (Gravitasi Kognitif).

```rust
pub struct MicroEntity {
    pub position: (f32, f32),
    pub color: u8,
    pub self_tensor: Array1<f32>,       // Identitas piksel ini
    pub neighborhood_tensor: Array1<f32>, // Superposisi lingkungan (LNS)
}
```

### B. Rumus Kuantum: Fractional Binding untuk Jarak

Jika piksel A (Merah) bersebelahan dengan piksel B (Biru) di koordinat (dx=1, dy=0), kita mendeskripsikan hubungan mereka dengan *Fractional Binding* FHRR:

```rust
// Pengaruh B ke A
let distance = 1.0;
let attenuation = (-distance / 2.0).exp(); // Decay eksponensial
let directional_tensor = FHRR::bind(&delta_x_1_tensor, &delta_y_0_tensor);
let b_influence = FHRR::bind(&b_semantic, &directional_tensor) * attenuation;

// LNS piksel A = A_self + b_influence + c_influence + ...
```

**Dampak:**
Agen sekarang bisa mengenali "Titik merah yang disebelah kanan-nya ada titik biru" murni dari menghitung L2 Distance / Cosine Similarity antara dua *neighborhood_tensor*. Tidak perlu `if/else` pixel checking!

---

## 🧬 2. RISET GENERATIF: Skill Composer & Tensor Interpolation

Jika MCTS mengalami *Catastrophic Failure* (semua *branch* hancur / `CognitiveMode::Counterfactual` terpicu), agen harus menciptakan **Skill Baru** yang tidak ada di *Ontology*.

### A. Quantum Crossover (Kombinasi Ide Gagal)

Misalkan MCTS memiliki 2 *WaveNode* yang mati di depth 1:
1. Node 1: Pragmatic Error 40.0 (Bisa memperbaiki sisi Kiri, tapi Kanan hancur). Axiom: `TRANS_X_5`
2. Node 2: Pragmatic Error 35.0 (Bisa memperbaiki sisi Kanan, tapi Kiri hancur). Axiom: `TRANS_Y_2`

`SkillComposer` melakukan *Crossover*:
```rust
// Kombinasikan dua tensor spasial dari ide yang gagal
let mut novel_spatial = FHRR::bind(&node1.tensor_spatial, &node2.tensor_spatial);

// Atau Interpolasi Superposisi (Fractional)
// 0.5 * TRANS_X_5 + 0.5 * TRANS_Y_2
let novel_spatial_superpos = (node1.tensor_spatial * 0.5) + (node2.tensor_spatial * 0.5);
FHRR::renormalize(&mut novel_spatial_superpos);
```

### B. Generative Conditional Tensor (IF-THEN Dinamis)

Agen menyadari bahwa "Axiom 1 hanya berlaku untuk warna Merah" dan "Axiom 2 hanya untuk Biru". Agen men- *generate* rule baru secara *on-the-fly*:

```rust
// Menciptakan Rule: IF (Piksel Merah) THEN (Gunakan Novel Spatial)
let conditional_red = FHRR::bind(&semantic_red, &novel_spatial);
```
Sistem MCTS (via *CounterfactualEngine*) kemudian menguji `conditional_red` ini. Jika berhasil menekan *Pragmatic Error*, ia permanen disimpan di `LogicSeedBank`.

---

## ⚙️ 3. IMPLEMENTASI RUST (Buku Hitam)

### Algoritma Micro-Relational Alignment (Branchless SoA)

Di dalam `TopologicalAligner` atau struktur baru `MicroRelationalEngine`:

```rust
pub fn compute_neighborhood_simd(
    pixels_x: &[f32], pixels_y: &[f32], pixels_color: &[i32],
    spatial_tensors: &[Array1<f32>], semantic_tensors: &[Array1<f32>],
    out_lns: &mut [Array1<f32>]
) {
    let n = pixels_x.len();

    // O(N^2) tapi N kecil (< 100) karena Fractional Manifold membatasi ukuran MicroLevel!
    for i in 0..n {
        out_lns[i].fill(0.0);

        for j in 0..n {
            if i == j { continue; }

            let dx = pixels_x[j] - pixels_x[i];
            let dy = pixels_y[j] - pixels_y[i];
            let dist_sq = dx*dx + dy*dy;

            if dist_sq > 9.0 { continue; } // Radius 3 piksel

            let attenuation = (-dist_sq.sqrt() * 0.5).exp();

            // FHRR bind branchless (Superposisi Semantic B + Arah Relatif)
            // Membutuhkan cache pre-computed directional tensors untuk kecepatan
            let influence = compute_directional_influence(
                &semantic_tensors[j], dx, dy, attenuation
            );

            for d in 0..GLOBAL_DIMENSION {
                out_lns[i][d] += influence[d];
            }
        }
        FHRR::renormalize(&mut out_lns[i]); // Kembalikan ke L2=1
    }
}
```

### Algoritma Generative Skill Synthesis (MCTS Fallback)

Di dalam `CeoDispatcher` atau `AsyncWaveSearch`, saat `ground_states.len() == 0`:

```rust
pub fn synthesize_novel_axioms(dead_waves: &[WaveNode]) -> Vec<WaveNode> {
    let mut novel_waves = Vec::new();

    // Sort dead waves by least pragmatic error
    // Ambil top 3
    let mut top_waves = dead_waves.to_vec();
    top_waves.sort_by(|a, b| a.pragmatic_error.partial_cmp(&b.pragmatic_error).unwrap());

    for i in 0..2 {
        for j in i+1..3 {
            let mut novel_tensor = Array1::zeros(GLOBAL_DIMENSION);
            // Fractional Binding (Crossover)
            for d in 0..GLOBAL_DIMENSION {
                novel_tensor[d] = top_waves[i].tensor_spatial[d] * 0.6 + top_waves[j].tensor_spatial[d] * 0.4;
            }
            FHRR::renormalize(&mut novel_tensor);

            let mut new_wave = top_waves[i].clone();
            new_wave.axiom_type.push("SYNTHESIZED_CROSSOVER".to_string());
            new_wave.tensor_spatial = novel_tensor;
            new_wave.depth += 1;

            novel_waves.push(new_wave);
        }
    }

    novel_waves
}
```

---

## 🎯 Dampak Strategis
1. **ARC Drawing/Connecting Tasks:** ARC sering meminta kita "tarik garis dari A ke B". Dengan *Neighborhood Signature*, MCTS bisa me- *resonansi* ujung garis dengan targetnya secara alami lewat tensor, tanpa aturan deteksi garis yang *hardcoded*.
2. **Generality:** Jika arsitektur V1/V2 adalah "Mencari dari list SkillOntology", arsitektur V3.1 ini adalah **"Menciptakan persamaan fisika baru saat alam semesta membutuhkannya."** Murni *Active Inference* tingkat tinggi!
