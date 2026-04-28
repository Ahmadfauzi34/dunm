# 🔬 MICRO-RELATION ENGINE (MRE) - RRM V3.2

`MicroRelationalEngine` adalah inti komputasi untuk memahami interaksi spasial tingkat piksel dalam arsitektur Fraktal RRM. MRE beroperasi di atas `InfiniteDetailField` (khususnya `MicroLevel`) dan bertujuan menemukan pola hubungan spasial secara "kuantum" tanpa menggunakan deteksi fitur hardcoded (`if/else`).

---

## 1. Konsep Utama: Local Neighborhood Signature (LNS)
Daripada mendeteksi "bentuk" atau "garis" secara geometris eksplisit, MRE menghitung **Local Neighborhood Signature (LNS)** untuk setiap piksel. LNS adalah superposisi dari identitas (warna/semantik) piksel-piksel tetangganya, dikalikan dengan bobot atenuasi eksponensial berdasarkan jarak, dan diikat (bound) dengan vektor arah (spatial tensor).

$$ \text{LNS}_i = \sum_{j \neq i} \left( \text{Sem}_j \otimes \text{Spat}(\mathbf{p}_j - \mathbf{p}_i) \right) \cdot e^{-\alpha \|\mathbf{p}_j - \mathbf{p}_i\|} $$

Di mana:
- $\text{Sem}_j$: Tensor Semantik dari piksel $j$ (Warna)
- $\text{Spat}(\Delta \mathbf{p})$: Tensor Spasial (Arah dari $i$ ke $j$)
- $\alpha$: Faktor atenuasi (decay rate)
- $\otimes$: FHRR Fractional Binding

---

## 2. Arsitektur Data (SoA & Zero-Copy)

MRE tidak menyimpan state sendiri, ia bertindak sebagai **Operator** yang membaca dari `InfiniteDetailField` menggunakan `ZeroCopyView`. MRE mem- *generate* tensor LNS yang bisa disimpan sementara dalam struktur SoA (Structure of Arrays).

```rust
pub struct MicroRelationalEngine {
    /// Jarak maksimum interaksi (cutoff untuk efisiensi komputasi)
    pub interaction_radius: f32,
    /// Seberapa cepat pengaruh tetangga memudar berdasarkan jarak
    pub decay_rate: f32,
}

impl MicroRelationalEngine {
    pub fn new(interaction_radius: f32, decay_rate: f32) -> Self {
        Self {
            interaction_radius,
            decay_rate,
        }
    }

    /// Menghitung LNS untuk sebuah MicroLevel
    /// Membutuhkan referensi ke MicroLevel (via Arc)
    pub fn compute_lns(&self, micro: &MicroLevel) -> Vec<Array1<f32>> {
        let n = micro.pixel_x.len();
        let mut lns_tensors = vec![Array1::zeros(GLOBAL_DIMENSION); n];

        // O(N^2) tapi N kecil (< 100) per MicroRegion
        for i in 0..n {
            if micro.pixel_mass[i] == 0.0 { continue; } // Skip ghost states

            let p_i = (micro.pixel_x[i], micro.pixel_y[i]);
            let mut lns_i = Array1::zeros(GLOBAL_DIMENSION);

            for j in 0..n {
                if i == j || micro.pixel_mass[j] == 0.0 { continue; }

                let p_j = (micro.pixel_x[j], micro.pixel_y[j]);
                let dx = p_j.0 - p_i.0;
                let dy = p_j.1 - p_i.1;
                let dist_sq = dx*dx + dy*dy;

                if dist_sq > self.interaction_radius * self.interaction_radius {
                    continue;
                }

                let dist = dist_sq.sqrt();
                let attenuation = (-self.decay_rate * dist).exp();

                // Dapatkan semantic tensor dari piksel J
                // (Optimasi: Di MicroLevel, kita bisa pre-compute Semantic Tensor berdasarkan warna)
                if let Some(sem_j) = micro.semantic_tensors.get(&j) {
                    // Buat tensor arah spasial fraksional (dx, dy)
                    let spatial_dx = FHRR::fractional_bind(&BaseSeeds::X, dx);
                    let spatial_dy = FHRR::fractional_bind(&BaseSeeds::Y, dy);
                    let directional_tensor = FHRR::bind(&spatial_dx, &spatial_dy);

                    // Pengaruh j ke i: Semantik J diikat dengan arah dari I ke J, diatenuasi
                    let mut influence = FHRR::bind(sem_j, &directional_tensor);
                    influence *= attenuation;

                    // Superposisikan ke LNS i
                    lns_i += &influence;
                }
            }

            // Normalisasi kembali ke L2=1 agar bisa digunakan untuk perhitungan similaritas
            FHRR::renormalize(&mut lns_i);
            lns_tensors[i] = lns_i;
        }

        lns_tensors
    }
}
```

---

## 3. Eksekusi Relasional: Pattern Matching
Dengan adanya LNS, agen dapat melakukan "Pencocokan Pola" (Pattern Matching) hanya dengan membandingkan LNS piksel di input dengan LNS piksel di output (Training Pairs).

Jika LNS Piksel $A$ di input mirip dengan LNS Piksel $A'$ di output (Cosine Similarity tinggi), maka secara topologis lingkungannya **identik**.

```rust
pub fn find_topological_analog(
    lns_source: &Array1<f32>,
    target_lns_array: &[Array1<f32>]
) -> Option<usize> {
    let mut best_sim = -1.0;
    let mut best_idx = None;

    for (i, target_lns) in target_lns_array.iter().enumerate() {
        let sim = FHRR::similarity(lns_source, target_lns);
        if sim > best_sim {
            best_sim = sim;
            best_idx = Some(i);
        }
    }

    if best_sim > 0.85 { // Threshold analogi
        best_idx
    } else {
        None
    }
}
```

---

## 4. Keunggulan Strategis MRE

1. **OOM-Proof & SIMD Friendly:** Algoritma ini murni operasi iterasi array dan perkalian matriks, menjadikannya sangat bersahabat dengan SIMD (Single Instruction, Multiple Data) dan terbebas dari GC (Garbage Collection).
2. **Kekebalan Terhadap Pergeseran Global (Shift-Invariant):** Karena LNS dihitung berdasarkan selisih relatif (`dx`, `dy`), jika sebuah pola (misalnya "kursor merah di tengah kotak biru") digeser 10 piksel ke kanan di *testing set*, LNS piksel kursor merah akan tetap bernilai **sama** dengan di *training set*. Ini sangat penting untuk memecahkan soal ARC!
3. **Lazy Evaluation:** Sesuai prinsip `InfiniteDetailField`, MRE hanya diinstansiasi dan menghitung tensor di level `Micro` jika MCTS merasa butuh. Jika task bisa diselesaikan di level `Macro` (misal memutar seluruh gambar), MRE tidak akan berjalan.
