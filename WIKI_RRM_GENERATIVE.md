# 🧬 WIKI-RRM: GENERATIF & MICRO-RELATION (RRM V3.1)

Melanjutkan fondasi **Fractal AI** (Makro-Meso-Mikro), riset ini menukik lebih dalam ke level **Mikro (Piksel)** dan **Generatif (Sintesis Skill)**. Menggeser fungsi MCTS statis dan *Hardcoded Skill Ontology* menjadi sistem Wiki terdesentralisasi yang secara proaktif men- *generate*, menganalisis, dan mendokumentasikan hukum fisikanya sendiri secara *on-the-fly*.

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
Jika piksel A (Merah) bersebelahan dengan piksel B (Biru) di koordinat (dx=1, dy=0):
```rust
// Pengaruh B ke A
let distance = 1.0;
let attenuation = (-distance / 2.0).exp(); // Decay eksponensial
let directional_tensor = FHRR::bind(&delta_x_1_tensor, &delta_y_0_tensor);
let b_influence = FHRR::bind(&b_semantic, &directional_tensor) * attenuation;

// LNS piksel A = A_self + b_influence + c_influence + ...
```
**Dampak:** Agen sekarang bisa mengenali "Titik merah yang disebelah kanan-nya ada titik biru" murni dari menghitung L2 Distance / Cosine Similarity antara dua `neighborhood_tensor`. Tidak perlu `if/else` pixel checking!

---

## 🧬 2. RISET GENERATIF: Skill Composer & Tensor Interpolation

Jika MCTS mengalami *Catastrophic Failure* (semua *branch* hancur / `CognitiveMode::Counterfactual` terpicu), agen harus menciptakan **Skill Baru** yang tidak ada di *Ontology*.

### A. Quantum Crossover (Kombinasi Ide Gagal)
Misalkan MCTS memiliki 2 *WaveNode* yang mati di depth 1:
1. Node 1: Pragmatic Error 40.0 (Bisa memperbaiki sisi Kiri, tapi Kanan hancur). Axiom: `TRANS_X_5`
2. Node 2: Pragmatic Error 35.0 (Bisa memperbaiki sisi Kanan, tapi Kiri hancur). Axiom: `TRANS_Y_2`

`SkillComposer` melakukan *Crossover*:
```rust
// Interpolasi Superposisi (Fractional)
// 0.5 * TRANS_X_5 + 0.5 * TRANS_Y_2
let novel_spatial_superpos = (node1.tensor_spatial * 0.5) + (node2.tensor_spatial * 0.5);
FHRR::renormalize(&mut novel_spatial_superpos);
```

### B. Generative Conditional Tensor (IF-THEN Dinamis)
Agen menyadari bahwa "Axiom 1 hanya berlaku untuk warna Merah". Agen men- *generate* rule baru secara *on-the-fly*:
```rust
// Menciptakan Rule: IF (Piksel Merah) THEN (Gunakan Novel Spatial)
let conditional_red = FHRR::bind(&semantic_red, &novel_spatial);
```
---

## 📚 3. INTEGRASI WIKI-RRM: Self-Documenting AI

Menggantikan `SkillOntology` berbasis *code*, kita mengadopsi direktori Wiki (`.md`). Saat MCTS melakukan *Crossover* dan berhasil, ia mendokumentasikan dirinya sendiri!

### A. Skill File Definition (`generated/micro/compute_lns.md`)
```markdown
---
id: compute_lns
type: micro
tier: 9
parent: topological_field
---
## Algorithm (Branchless SoA)
```rust
pub fn compute_lns(positions: &[(f32, f32)], semantics: &[FHRR], radius: f32) -> Vec<FHRR> { ... }
```

### B. Git-Style Branching di "Mental Space" (Metakognisi Visual)
Di dalam pikirannya (`CounterfactualEngine` / `MentalReplay`), setiap eksperimen `Skill` baru bukan sekadar log, tetapi beroperasi seperti sebuah Branching Git Tree:

```text
## Execution Log
### [t0] Run #1 → SUCCESS

## Analysis Log
### [t1] Analysis: "Could be better"

## Branch: Experiment_A  ← RRM fork skill!
### [t2] Patch: Aggressive optimization
### [t3] Run #2 → FAIL (too aggressive)

## Branch: Experiment_B  ← Fork lain!
### [t2] Patch: Conservative optimization
### [t3] Run #2 → SUCCESS +5% speed
### [t4] MERGE to main  ← Promote yang bagus
```
Sistem ini menggunakan Markdown untuk melacak dan memvisualisasikan `Lineage` (keturunan) *Skill*. Setiap kali MCTS "Mentok", ia melakukan *Git Checkout* ke percabangan baru untuk men-*sintesis* aksioma, lalu melakukan *Git Merge* ke main knowledge base jika hasil Evaluasi Energinya (`Pragmatic Error == 0`) terbukti *SUCCESS*.

### C. MCTS Fallback Wiki Generator
```rust
impl GenerativeWiki {
    pub fn on_catastrophic_failure(&mut self, dead_waves: &[WaveNode]) {
        let analysis = self.analyze_failure_patterns(dead_waves);
        let novel_axioms = self.synthesize_novel_axioms(dead_waves);

        for (i, novel) in novel_axioms.iter().enumerate() {
            let skill_id = format!("synthesized_{}_{}", chrono::Utc::now(), i);
            let skill_md = self.generate_skill_markdown(&skill_id, novel, &analysis, &dead_waves);

            // Auto-document the synthesis!
            self.wiki.create_skill(&skill_id, skill_md);
        }
        self.mcts.inject_novel_axioms(novel_axioms);
    }
}
```

---

## 🎯 Kesimpulan: AGI yang "Bisa Dijelaskan" (Explainable)

Jika RRM V1/V2 menggunakan "Aturan yang diajarkan manusia", V3.1 "Menciptakan aturan fisika kosmik baru saat alam semesta membutuhkannya."

> RRM (Micro-Relation + Generatif) adalah 'engine' otot kuantum. Wiki-RRM adalah 'memory dan interface'. Bersama mereka menciptakan sistem yang tidak hanya pintar, tapi juga mengerti dan menjelaskan kebodohannya (lewat Git-branching `.md`)—lalu belajar darinya.

Dengan ini AI tidak lagi bertindak sebagai "Black-Box". Setiap intuisi yang didapat terekam dalam format Wiki yang *human-readable*, mendobrak batas algoritma yang kaku dan buram.
