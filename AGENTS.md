# Protocol Analisis Mendalam (Deep Analysis Protocol) untuk RRM Quantum Sandbox

Sebagai agen AI (Bolt/Jules), Anda **DIWAJIBKAN** melakukan proses berpikir (Internal Discussion / Chain of Thought) yang mendalam sebelum mengedit kode inti (core logic), terutama pada arsitektur kompleks berkinerja tinggi seperti `EntityManifold` (SoA), `MultiverseSandbox`, `QuantumSearch`, dan `WaveDynamics`.

Repositori `rrm_rust` menggunakan pendekatan **Zero-Cost Abstractions**, **Copy-on-Write (CoW)**, dan komputasi termodinamika berorientasi Tensor. Patuhi aturan berikut dengan ketat dan jangan pernah melewatinya (no bypass):

## 0. Konfigurasi Lint & Build (Hukum yang Sudah Terkodifikasi)

Konfigurasi berikut **SUDAH AKTIF** di codebase dan tidak boleh dimodifikasi agen tanpa persetujuan eksplisit:

### `src/lib.rs` (Lint Attributes)
```rust
#![cfg_attr(not(test), warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::perf,
    clippy::complexity,
    clippy::style,
))]
#![cfg_attr(not(test), deny(
    clippy::correctness,
    clippy::suspicious,
))]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
)]
```

### `.cargo/config.toml` (Build & Alias)
```toml
[build]
rustflags = [
    "-C", "target-cpu=native",
    "-Dwarnings",
]

[alias]
clippy-strict = [
    "clippy", "--",
    "-W", "clippy::all",
    "-W", "clippy::pedantic",
    "-W", "clippy::nursery",
    "-W", "clippy::cargo",
    "-W", "clippy::perf",
    "-W", "clippy::complexity",
    "-W", "clippy::style",
    "-D", "clippy::correctness",
    "-D", "clippy::suspicious",
    "-A", "clippy::module_name_repetitions",
    "-A", "clippy::must_use_candidate",
]
```

> **Catatan Penting:** Lint di `lib.rs` sudah cukup untuk `cargo clippy`. Alias `clippy-strict` ada sebagai fallback. Agen tidak perlu menambah lint attribute baru di file individual.

---

## 1. Fase Pemahaman (Grasp Before Act)
- **Dilarang langsung mengedit file** saat menerima laporan bug berbasis logika atau performa.
- Gunakan perintah pencarian (`grep` via bash) atau baca file yang terkait secara menyeluruh menggunakan `cat`/`read_file` sebelum berasumsi.
- **Pahami Siklus Memori dan Tensor:**
  - Jika Anda memodifikasi entitas (`EntityManifold`), ingat bahwa ini adalah *Structure of Arrays (SoA)* berbasis `Vec<T>`.
  - Iterasi data berpusat pada perataan memori cache (L1/L2 locality).
  - Pahami bagaimana *Deep Copy* dan *Shallow Clone* bekerja di dalam `MultiverseSandbox` (bagaimana memori disalin antar dimensi probabilitas).

## 2. Aturan Emas Performa (Speed is a Feature, Memory is a Resource)
- **No Heap Thrashing:** Dilarang keras menempatkan alokasi memori (seperti `Vec::new()`, `clone()`, atau `.to_string()`) di dalam *hot loop* seperti MCTS (`quantum_search.rs`) atau perambatan gelombang. Gunakan pendekatan in-place mutation atau *object pool*.
- **No False Sharing:** Hindari mekanisme *locking* atau sinkronisasi atomic (`Arc`, `Mutex`) pada level piksel/entitas tunggal yang dapat menyebabkan kemacetan bus data pada CPU.
- **SIMD/Vectorization Readiness:** Tulis loop iterasi sedatar dan sejelas mungkin agar *compiler* LLVM (Rust) dapat melakukan *Auto-Vectorization*. Hindari *branching* (`if-else`) di dalam iterasi *tensor math*.
- **Branchless Math in Tensor Kernels:** Perbandingan float di dalam loop tensor harus menggunakan `copysign`, `clamp`, atau bitmask — bukan `if-else`. Gunakan epsilon constant (`1e-6`) untuk zero-guard, bukan exact equality (`==`).
- **Pre-Allocated Buffer Reuse:** Setiap komponen SoA yang membutuhkan temporary storage (misal: accumulation buffer, scratch pad) harus di-pre-allocate di `MultiverseSandbox` dan direuse antar frame. `Vec::new()` di dalam `update()` atau `query()` adalah pelanggaran berat.

## 3. Disiplin Memori & Ownership (Memory Discipline)
- **No Shift in SoA:** Dilarang menggunakan `Vec::remove`, `Vec::retain`, atau operasi yang menyebabkan element shift di dalam `EntityManifold`. Gunakan **swap-remove dengan tombstone (ghost state)** atau pre-allocated sparse set. **Index stability adalah hukum.**
- **Ghost States / Swap-Drop:** Alih-alih menghapus elemen dari array (yang mem-shift indeks dan invalidasi referensi), gunakan pattern ghost state: tandai elemen sebagai `dead` via boolean flag, lalu kompakkan di fase akhir frame dengan swap-remove batch. Jangan pernah pakai `Vec::remove` di hot path.
- **Zero-Cost Dispatch Only:** Selalu prefer **enum dispatch** (algebraic data types) atau static monomorphization (`impl Trait` / generic). `Box<dyn Trait>` dilarang di hot path kecuali untuk initialization phase. Virtual dispatch menghalangi inlining dan SIMD.
- **Field-Based Computation Only:** Entity tidak boleh di-spawn/kill secara diskrit di tengah frame. Perubahan state harus melalui **kernel convolution** atau **wave interference** pada field tensor. Spawn/kill hanya diizinkan di fase boundary (frame start/end) dengan batching.

## 4. Pengecekan Edge Case (Boundary Analysis)
Sebelum merubah algoritma matematika FHRR, MCTS, atau topologi spasial, Anda harus mengevaluasi:
- **Zero/Empty State:** Apa yang terjadi jika array panjangnya 0? Bagaimana jika `active_count` bernilai 0? (Contoh: Menghindari pembagian dengan nol saat normalisasi L2).
- **Infinite/NaN State:** Operasi gelombang (*phase rotation*, trigonometri) rentan terhadap *floating point traps*. Selalu periksa nilai tak terhingga (Infinite/NaN).
- **Out of Bounds:** Saat memotong (*cropping*) dimensi *Multiverse*, pastikan perhitungan `min_x`, `max_y` tidak menghasilkan indeks negatif atau melampaui ukuran matriks arena.

## 5. Dokumentasi Rencana & Clippy
- Jika Anda harus memperbaiki arsitektur logika yang berdampak besar, gunakan `set_plan` untuk menjabarkan:
  1. Akar masalah (Root cause).
  2. Mengapa pendekatan sebelumnya gagal (misal: *heap thrashing*).
  3. Konsekuensi dari perbaikan baru (Cascade effect).
- **Linter Adalah Hukum:** Anda wajib menjalankan `cargo clippy` (atau `cargo clippy-strict`) jika membuat file baru atau memodifikasi secara ekstensif. Jangan biarkan peringatan *pedantic* menumpuk!

## 6. Disiplin Testing (Micro-benchmarking)
- Setelah mengubah *core logic*, Anda WAJIB memverifikasinya.
- Jalankan `cargo test --release` untuk memastikan tes terdistribusi (seperti tes termodinamika CoW) tidak gagal (*FAIL*).
- Jalankan *micro-benchmark* (`cargo run --release --bin bench_topology`) jika optimasi memengaruhi *EntityManifold* atau matriks struktural untuk membuktikan kecepatan eksekusi (waktu rerata).

## 7. Pre-Flight Checklist (Agen Tidak Boleh Bypass)
Sebelum mengakhiri sesi atau melaporkan "selesai", agen WAJIB menjalankan dan melampirkan output:
1. `cargo clippy --all-targets --all-features` (harus **0 error, 0 warning**).
2. `cargo test --release` (semua **PASS**).
3. `cargo run --release --bin bench_topology` (tidak ada regresi performa >5%).
4. Verifikasi tidak ada file baru yang belum didaftarkan di `lib.rs` / `Cargo.toml`.
5. Update `.jules/engineering_journal.md` dengan signature `[⬡ Carbo]` atau `[⚡ Bolt]` sesuai peran.

> **Perhatian:** Jika salah satu poin di atas gagal, agen **TIDAK BOLEH** melaporkan tugas sebagai selesai. Perbaiki terlebih dahulu.

---
*Catatan: Dokumen ini dimuat secara otomatis oleh sistem sebagai instruksi dasar khusus untuk proyek RRM (Recursive Relationship Modeling). Anda tidak memiliki izin untuk mengabaikan instruksi yang berfokus pada arsitektur presisi memori ini.*

## 8. Shared Engineering Journal (Nexus)
Sebelum memulai tugas apa pun, Anda **WAJIB** membaca file `.jules/engineering_journal.md` (buat jika belum ada). File ini adalah *shared nexus* antara Anda (Carbo ⬡ / Architect) dan agen pengoptimal performa (Bolt ⚡). Membaca catatan kritis dari Bolt akan mencegah Anda merancang solusi yang dapat menyebabkan kebocoran memori atau hambatan performa.

Anda hanya boleh menambahkan catatan ke **Section 2 (Architectural Decisions)** dan **Section 3 (Future Ideas)**.
**Anda WAJIB menandatangani entri jurnal Anda dengan `[⬡ Carbo]`.**

**Format Examples:**

*Section 2 (Architectural Decisions):*
`## YYYY-MM-DD - [⬡ Carbo] - [Component/Architecture Name]`
**Context:** [Masalah yang sedang dipecahkan]
**Decision:** [Pendekatan kreatif/arsitektural yang dipilih]
**Consequences:** [Trade-offs, misal: "Kode lebih elegan, tapi perhatikan alokasi memori"]

*Section 3 (Future Ideas):*
`## YYYY-MM-DD - [⬡ Carbo] - [Creative/Architectural Idea Title]`
**Vision:** [Struktur luar biasa apa yang ingin Anda bangun]
**Blockers:** [Mengapa tidak diimplementasikan hari ini]
