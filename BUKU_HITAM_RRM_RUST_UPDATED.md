# 📜 BUKU HITAM ARSITEKTUR RRM - RUST EDITION
**Katalog Dosa-Dosa Performa & Solusi High-Dimensional Computing dalam Rust**

> Versi: 2.0-Rust  
> Domain: VSA/FHRR, Entity Component System, Creative Computing  
> Optimized for: x86_64/AArch64, SIMD, Cache-Friendly Architecture

---

## 🎯 FILOSOFI UTAMA

Rust memungkinkan zero-cost abstraction, tapi **zero-cost tidak otomatis**. Dosa-dosa berikut adalah pattern yang **mengorbankan performa** meski tetap "safe" di mata borrow checker.

> *"Compile tanpa error ≠ Run tanpa penalty"*

---

## 🚫 DOSA 1: "Closure Allocation di Hot Path"

### Bentuk Dosa
```rust
// Dosa: Closure allocation per iterasi
for i in 0..1_000_000 {
    let condition = some_check();
    condition.then(|| {
        // 100 baris logic...
        expensive_computation();
    });
}
```

### Kenapa Menghancurkan Mesin?
- `then()` menerima `FnOnce`, forcing **heap allocation** untuk closure capture
- **Branch predictor** gagal karena indirection function pointer
- **Cache pollution**: Closure code tidak inline, melompat ke alamat acak

### Penebusan Dosa (Branchless Control Flow)
```rust
// ✅ Benar: Explicit branch dengan early continue
for i in 0..1_000_000 {
    if !some_check() { continue; }
    // 100 baris logic - inline, cache-friendly
    expensive_computation();
}

// ✅ Alternatif: Boolean mask untuk SIMD-style (jika tanpa side effect)
let mask = some_check() as usize;
result[i] = value * mask; // 0 jika false, value jika true
```

### Kapan Boleh Melanggar?
- **Cold path** (< 1% execution time): Readability > micro-optimasi
- **Non-SIMD loop** dengan complexity rendah

---

## 🚫 DOSA 2: "Vec<T> di dalam Struct (AOS)"

### Bentuk Dosa
```rust
// Dosa: Array of Structs - cache thrashing
#[derive(Clone)]
struct Entity {
    position: Vec3,
    velocity: Vec3,
    mass: f32,
    tensor: [f32; 8192], // Large, scattered
}

struct World {
    entities: Vec<Entity>, // Pointer chasing nightmare
}
```

### Kenapa Menghancurkan Mesin?
- `Entity` ukuran besar (> 32KB dengan tensor), **sparse di heap**
- Akses `entities[i].tensor[j]` = **cache miss** tiap iterasi
- **False sharing**: Multi-threaded access ke field berbeda di struct sama

### Penebusan Dosa (Structure of Arrays / ECS)
```rust
// ✅ Benar: SOA - cache line packing
pub struct EntityManifold {
    // Hot data (frequently accessed together)
    pub centers_x: Vec<f32>,
    pub centers_y: Vec<f32>,
    pub masses: Vec<f32>,
    pub tokens: Vec<i32>,

    // Cold data (sparse access)
    pub spatial_tensors: Vec<FHRR>,  // Only when VSA operation needed
    pub semantic_tensors: Vec<FHRR>,

    // Metadata
    pub active_count: usize,
    pub capacity: usize,
}

// ✅ Access pattern: Linear scan, 100% cache hit
for i in 0..manifold.active_count {
    if manifold.masses[i] == 0.0 { continue; }
    // centers_x[i] dan centers_y[i] di cache line yang sama
    process(manifold.centers_x[i], manifold.centers_y[i]);
}
```

### Kapan Boleh Melanggar?
- **Entity count < 100**: AOS lebih readable, overhead minimal
- **Complex lifetime relationship**: Graph-like structure (gunakan `petgraph`)

---

## 🚫 DOSA 3: "FHRR Inverse Total"

### Bentuk Dosa
```rust
// Dosa: Inverse tensor superposisi menghancurkan semantik warna
let mirrored = fhrr::inverse(&entity_tensor);
// entity_tensor = X ⊗ Y ⊗ Color
// inverse() membalik X, Y, DAN Color → "Anti-Color"
```

### Kenapa Menghancurkan Semesta?
- Tensor VSA adalah **binding komutatif**: `A ⊗ B ⊗ C`
- Inverse total = `A⁻¹ ⊗ B⁻¹ ⊗ C⁻¹`
- **Color phase** terbalik, decoder tidak mengenali (render hitam/artifact)

### Penebusan Dosa (Calibrated Fractional Translation)
```rust
/// Inversi spasial murni tanpa sentuh semantik
pub fn mirror_spatial(tensor: &FHRR, axis: Axis, center: f32) -> FHRR {
    // Unbind komponen spasial saja
    let (spatial, semantic) = unbind_components(tensor);

    // Transform spasial: reflection = translation terkalibrasi
    let reflected = match axis {
        Axis::X => fractional_bind(&X_SEED, 2.0 * center - extract_x(&spatial)),
        Axis::Y => fractional_bind(&Y_SEED, 2.0 * center - extract_y(&spatial)),
    };

    // Rebind dengan semantik utuh
    bind(&reflected, &semantic)
}

// ✅ Penggunaan: Warna tetap, posisi terbalik
let mirrored = mirror_spatial(&entity, Axis::X, 15.5);
```

### Kapan Boleh Melanggar?
- **Sengaja ingin "negative" entity**: Gunakan `inverse_explicit()` dengan dokumentasi jelas
- **Debug/visualisasi phase**: Inverse untuk melihat interference pattern

---

## 🚫 DOSA 4: "Vec::remove() / retain() di Loop"

### Bentuk Dosa
```rust
// Dosa: O(N²) shift, memory copy massive
for i in (0..entities.len()).rev() {
    if entities[i].is_dead() {
        entities.remove(i); // Shift semua elemen kanan!
    }
}
```

### Kenapa Menghancurkan Mesin?
- `remove(i)` = **memmove** semua elemen `i+1..len` ke kiri
- Untuk 10K entities × 8KB tensor = **80MB memory copy** per removal
- **Iterator invalidation**: Rust borrow checker melarang, tapi logic error tetap bisa

### Penebusan Dosa (Ghost States / Swap-Drop)
```rust
// ✅ Metode 1: Ghost States (Mass = 0.0)
pub struct EntityManifold {
    masses: Vec<f32>, // 0.0 = slot kosong (Dark Matter)
    // ...
}

impl EntityManifold {
    pub fn kill_entity(&mut self, idx: usize) {
        self.masses[idx] = 0.0; // O(1), no shift
        // Optional: push idx ke free_list untuk reuse
    }

    pub fn iter_active(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.active_count)
            .filter(|&i| self.masses[i] > 0.0)
    }
}

// ✅ Metode 2: Swap-Drop (untuk benar-benar hapus tapi O(1))
pub fn remove_swap(&mut self, idx: usize) {
    self.active_count -= 1;
    if idx != self.active_count {
        // Swap dengan terakhir, lalu drop
        self.centers_x.swap(idx, self.active_count);
        self.centers_y.swap(idx, self.active_count);
        // ... swap semua field
    }
}
```

### Kapan Boleh Melanggar?
- **Order preservation critical**: Gunakan `retain()` jika memang perlu maintain index
- **Small Vec (< 100)**: Overhead shift minimal

---

## 🚫 DOSA 5: "Branch di Math Loop (SIMD Killer)"

### Bentuk Dosa
```rust
// Dosa: Branch di dalam kalkulasi vektor
for i in 0..DIMENSION {
    let mag = (sum_sq).sqrt();
    if mag > 0.0 {
        tensor[i] /= mag;
    }
    // else: implicit 0, tapi branch tetap ada!
}
```

### Kenapa Menghancurkan Mesin?
- **SIMD auto-vectorization gagal**: LLVM tidak bisa vectorize loop dengan branch
- **Pipeline stall**: CPU harus flush jika prediksi salah
- **1e-15 vs 0.0**: Secara fisik identik, tapi satu branchless

### Penebusan Dosa (Epsilon + Branchless Math)
```rust
// ✅ Benar: Math branchless dengan epsilon
let mag_sq: f32 = tensor.iter().map(|x| x * x).sum();
let inv_mag = 1.0 / (mag_sq.sqrt() + 1e-15); // Epsilon prevents div by zero

// SIMD-friendly: pure arithmetic, no branch
for i in 0..DIMENSION {
    tensor[i] *= inv_mag;
}

// ✅ Advanced: Select intrinsic untuk conditional tanpa branch
use std::arch::x86_64::*;
let mask = _mm256_cmp_ps(ymm_mag, _mm256_setzero_ps(), _CMP_NEQ_OQ);
let result = _mm256_blendv_ps(ymm_zero, ymm_normalized, mask);
```

### Kapan Boleh Melanggar?
- **Domain requires exact 0 handling**: Gunakan `if` dengan `#[inline(never)]` untuk cold path
- **Non-performance critical**: Clarity > speed untuk setup/init code

---

## 🚫 DOSA 6: "Magic Number / -Infinity di Branchless Max"

### Bentuk Dosa
```rust
// Dosa: -Infinity × 0 = NaN (IEEE 754 trap)
let mut best = f32::NEG_INFINITY;
for i in 0..n {
    let val = compute_similarity(i);
    let is_better = (val > best) as i32 as f32;
    // Iterasi 1: NEG_INFINITY * 0.0 = NaN!
    best = best * (1.0 - is_better) + val * is_better;
}
```

### Kenapa Menghancurkan Mesin?
- **NaN propagation**: `NaN + x = NaN`, `NaN > x = false`
- **Silent failure**: Tidak panic, hanya hasil salah
- **Debug nightmare**: Muncul di iterasi ke-1000+, sulit trace

### Penebusan Dosa (Sensible Bounds + Type Safety)
```rust
// ✅ Benar: Sensible bound berdasarkan domain
pub const MIN_SIMILARITY: f32 = -1.0; // FHRR similarity range [-1, 1]
pub const MAX_SIMILARITY: f32 = 1.0;

// ✅ Lebih baik: Newtype dengan invariant
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Similarity(f32);

impl Similarity {
    pub const MIN: Self = Self(-1.0);
    pub const MAX: Self = Self(1.0);

    pub fn new_clamped(v: f32) -> Self {
        Self(v.clamp(-1.0, 1.0))
    }

    pub fn get(&self) -> f32 { self.0 }
}

// ✅ Branchless max yang aman
let mut best = Similarity::MIN;
for i in 0..n {
    let val = Similarity::new_clamped(compute_similarity(i));
    if val > best { best = val; } // OK: explicit branch, bukan math trick
    // Atau: best = best.max(val) - Rust intrinsic, optimized
}
```

### Kapan Boleh Melanggar?
- **Guaranteed non-empty iterator**: `Iterator::max()` handle ini untuk Anda
- **F64 precision**: `-1e308` masih aman (tetap hindari INFINITY)

---

## 🚫 DOSA 7: "Box<dyn Trait> di Hot Path" (Rust-Specific)

### Bentuk Dosa
```rust
// Dosa: Dynamic dispatch di loop kritis
trait Axiom { fn apply(&self, world: &mut World); }

struct Engine {
    axioms: Vec<Box<dyn Axiom>>, // vtable pointer chasing
}

impl Engine {
    fn update(&mut self, world: &mut World) {
        for axiom in &self.axioms {
            axiom.apply(world); // Indirect call, cache miss
        }
    }
}
```

### Kenapa Menghancurkan Mesin?
- **Vtable lookup**: Dereference pointer + indirect call
- **Monomorphization fail**: Code tidak inline, optimizer buta
- **Cache thrashing**: `dyn Axiom` tersebar di heap

### Penebusan Dosa (Enum Dispatch / Static Dispatch)
```rust
// ✅ Benar: Enum dispatch - tag + union, cache-friendly
#[derive(Clone)]
pub enum Axiom {
    Translate { dx: f32, dy: f32 },
    Rotate { angle: f32 },
    Spawn { color: i32, count: usize },
    Crop { quadrant: u8 },
    // ... tier 0-8
}

impl Axiom {
    pub fn apply(&self, world: &mut World) {
        match self {
            Self::Translate { dx, dy } => world.translate(*dx, *dy),
            Self::Rotate { angle } => world.rotate(*angle),
            // ... exhaustive, no vtable
        }
    }
}

// ✅ Advanced: Generic dengan const generic untuk tier
pub struct Engine<const TIER: u8> {
    axioms: Vec<Axiom>, // Monomorphized per tier
}
```

### Kapan Boleh Melanggar?
- **Plugin system external**: `dyn` untuk DLL hot-reload
- **Truly heterogeneous**: Enum variant > 50, consider `dyn`

---

## 🚫 DOSA 8: "Iterator Chain Complex tanpa .collect()"

### Bentuk Dosa
```rust
// Dosa: Nested iterator, compiler tidak bisa optimize
let result: Vec<_> = entities.iter()
    .filter(|e| e.mass > 0.0)
    .map(|e| expensive_transform(e))
    .filter(|e| e.energy > threshold)
    .map(|e| another_transform(e))
    .collect();
```

### Kenapa Menghancurkan Mesin?
- **State machine overhead**: Tiap adapter = struct + next() call
- **No SIMD**: LLVM tidak melihat loop utuh
- **Branch mispredict**: Filter terpisah, tidak fused

### Penebusan Dosa (Explicit Loop dengan Fused Logic)
```rust
// ✅ Benar: Single loop, fused branches
let mut result = Vec::with_capacity(entities.len());
for e in &entities {
    if e.mass <= 0.0 { continue; }

    let transformed = expensive_transform(e);
    if transformed.energy <= threshold { continue; }

    result.push(another_transform(&transformed));
}

// ✅ Atau: Iterator + fold untuk reduksi (tetap efisien)
let total_energy: f32 = entities.iter()
    .filter(|e| e.mass > 0.0)
    .map(|e| e.energy)
    .sum(); // Sum adalah reduksi, tetap optimal
```

### Kapan Boleh Melanggar?
- **Clarity > speed**: Business logic, bukan hot path
- **Lazy evaluation penting**: Iterator chain untuk infinite stream

---

## 🚫 DOSA 9: "Mutex di Loop Paralel"

### Bentuk Dosa
```rust
// Dosa: Lock contention menghancurkan paralelisme
use std::sync::{Arc, Mutex};

let shared = Arc::new(Mutex::new(Vec::new()));

(0..1000).into_par_iter().for_each(|i| {
    let result = compute(i);
    shared.lock().unwrap().push(result); // Serial bottleneck!
});
```

### Kenapa Menghancurkan Mesin?
- **False sharing**: Mutex cache line bouncing antar core
- **Serialisasi**: Paralelisme → sequential karena lock
- **Poison risk**: Panic di thread = mutex poisoned

### Penebusan Dosa (Lock-Free / Per-Thread Accumulator)
```rust
use rayon::prelude::*;

// ✅ Benar: Per-thread collection, merge akhir
let results: Vec<_> = (0..1000)
    .into_par_iter()
    .map(|i| compute(i)) // Tanpa lock, murni paralel
    .collect(); // Rayon handle merge efisien

// ✅ Atau: Crossbeam channel untuk stream
use crossbeam::channel;

let (sender, receiver) = channel::unbounded();

scope(|s| {
    s.spawn(|_| {
        for result in receiver {
            process(result); // Single consumer
        }
    });

    (0..1000).into_par_iter().for_each(|i| {
        sender.send(compute(i)).unwrap(); // Non-blocking send
    });
});
```

### Kapan Boleh Melanggar?
- **Critical section sangat singkat**: `Mutex<f32>` untuk counter (tetap hindari)
- **Single-threaded context**: `RefCell` untuk interior mutability

---

## 🚫 DOSA 10: "Allocation di Loop (Vec::with_capacity Fail)"

### Bentuk Dosa
```rust
// Dosa: Re-allocation per iterasi
for i in 0..1000 {
    let mut temp = Vec::new(); // Alloc 0, grow 1, 2, 4, 8...
    for j in 0..100 {
        temp.push(compute(i, j)); // 7-8 reallocation!
    }
    process(temp);
}
```

### Kenapa Menghancurkan Mesin?
- **Allocator pressure**: `malloc/free` ribuan kali
- **Memory fragmentation**: Heap tidak contiguous
- **Drop overhead**: `Vec` di-scope exit = dealloc

### Penebusan Dosa (Pre-allocated Buffer + Clear)
```rust
// ✅ Benar: Reuse buffer dengan clear
let mut temp = Vec::with_capacity(100); // Single alloc

for i in 0..1000 {
    temp.clear(); // O(1), tidak dealloc
    temp.extend((0..100).map(|j| compute(i, j)));
    process(&temp);
}

// ✅ Advanced: Bump allocator untuk frame-based
use bumpalo::Bump;

let bump = Bump::new();
for i in 0..1000 {
    let temp: &mut Vec<_> = bump.alloc(Vec::with_capacity(100));
    // ... use temp
    bump.reset(); // O(1) reset semua allocation
}
```

### Kapan Boleh Melanggar?
- **One-shot operation**: Function dipanggil sekali, tidak loop
- **Small fixed size**: `SmallVec<[T; 8]>` untuk stack allocation

---

## 🛠️ TOOLING: Deteksi Dosa Otomatis

### Clippy Lints (Wajib Aktif)
```toml
# .clippy.toml atau Cargo.toml
[lints.clippy]
all = "warn"
nursery = "warn"
perf = "deny"
correctness = "deny"

# Specific untuk dosa-dosa di atas
vec_box = "deny"           # Dosa 7
linkedlist = "deny"        # Cache unfriendly
mutex_integer = "deny"     # Dosa 9
redundant_clone = "warn"
```

### Custom Lint (Miri + Proptest)
```rust
#[test]
fn test_no_nan_branchless() {
    // Proptest: Pastikan branchless max tidak produce NaN
    proptest!(|(vals: Vec<f32>)| {
        let mut best = -999.0f32;
        for v in vals {
            let is_better = (v > best) as i32 as f32;
            best = best * (1.0 - is_better) + v * is_better;
        }
        prop_assert!(!best.is_nan());
    });
}
```

---

## 📊 CHEAT SHEET: Kapan Melanggar?

| Dosa | Hot Path (>90% time) | Warm Path | Cold Path |
|------|---------------------|-----------|-----------|
| 1. Closure | ❌ NEVER | ⚠️ Avoid | ✅ OK |
| 2. AOS | ❌ NEVER | ⚠️ Small N | ✅ OK |
| 3. FHRR Inverse | ❌ NEVER | ❌ NEVER | ⚠️ Explicit |
| 4. Vec::remove | ❌ NEVER | ⚠️ Small N | ✅ OK |
| 5. Math Branch | ❌ NEVER | ⚠️ Avoid | ✅ OK |
| 6. Magic Number | ❌ NEVER | ❌ NEVER | ❌ NEVER |
| 7. dyn Trait | ❌ NEVER | ⚠️ Avoid | ✅ OK |
| 8. Iterator Chain | ⚠️ Caution | ✅ OK | ✅ OK |
| 9. Mutex | ❌ NEVER | ⚠️ Careful | ⚠️ Careful |
| 10. Alloc Loop | ❌ NEVER | ⚠️ Avoid | ✅ OK |

---


---

## 🔄 ALTERNATIF: Non-SOA Statis dengan OOM Protection

SOA statis (fixed capacity) adalah **default yang aman**, tapi ada situasi di mana fleksibilitas dinamis diperlukan tanpa risiko OOM. Berikut pattern **"Controlled Dynamism"**:

### 1. Arena Allocator dengan Budget

```rust
use bumpalo::Bump;

pub struct BudgetedArena {
    arena: Bump,
    budget_bytes: usize,
    used_bytes: usize,
}

impl BudgetedArena {
    pub fn new(budget_mb: usize) -> Self {
        Self {
            arena: Bump::new(),
            budget_bytes: budget_mb * 1024 * 1024,
            used_bytes: 0,
        }
    }

    pub fn alloc<T>(&mut self, value: T) -> Option<&mut T> {
        let size = std::mem::size_of::<T>();
        if self.used_bytes + size > self.budget_bytes {
            return None; // Graceful degradation, bukan OOM
        }

        let ptr = self.arena.alloc(value);
        self.used_bytes += size;
        Some(ptr)
    }

    pub fn reset(&mut self) {
        self.arena.reset();
        self.used_bytes = 0;
    }
}

// Usage: Frame-based allocation dengan hard limit
fn process_frame(arena: &mut BudgetedArena, entities: &[Entity]) {
    for e in entities {
        // Alloc bisa gagal, handle gracefully
        let processed = match arena.alloc(process(e)) {
            Some(p) => p,
            None => {
                log::warn!("Arena budget exceeded, skipping remaining");
                break; // Soft failure
            }
        };
    }
    // Semua alloc di-reset di frame berikutnya
}
```

**Kelebihan:**
- **O(1) alloc/dealloc**: Bump pointer, tidak seperti system allocator
- **No fragmentation**: Sequential allocation, compact memory
- **OOM-proof**: Hard budget, graceful degradation
- **Cache-friendly**: Data tetap contiguous dalam arena

---

### 2. Generational Index + Sparse Set

```rust
/// Sparse Set: Dense data + Sparse index (hybrid SOA/fleksibel)
pub struct SparseSet<T> {
    // Dense: Data contiguous untuk iteration (SOA-like)
    dense: Vec<T>,

    // Sparse: Index mapping ID -> dense position
    sparse: Vec<Option<u32>>, // Index into dense

    // Generational counter untuk ABA safety
    generations: Vec<u32>,
    free_list: Vec<u32>, // Reusable sparse indices
}

pub struct Handle {
    index: u32,
    generation: u32,
}

impl<T> SparseSet<T> {
    pub fn insert(&mut self, value: T) -> Handle {
        let dense_idx = self.dense.len() as u32;
        self.dense.push(value);

        let sparse_idx = self.free_list.pop()
            .unwrap_or_else(|| {
                self.sparse.push(None);
                self.generations.push(0);
                (self.sparse.len() - 1) as u32
            });

        self.sparse[sparse_idx as usize] = Some(dense_idx);

        Handle {
            index: sparse_idx,
            generation: self.generations[sparse_idx as usize],
        }
    }

    pub fn get(&self, handle: Handle) -> Option<&T> {
        // Generational check: handle masih valid?
        if self.generations.get(handle.index as usize)? != handle.generation {
            return None; // Stale handle
        }

        let dense_idx = self.sparse[handle.index as usize]?;
        self.dense.get(dense_idx as usize)
    }

    pub fn remove(&mut self, handle: Handle) -> Option<T> {
        // Swap-remove dari dense (O(1))
        let sparse_idx = handle.index as usize;
        let dense_idx = self.sparse[sparse_idx]? as usize;

        let last = self.dense.len() - 1;
        self.dense.swap(dense_idx, last);
        let removed = self.dense.pop()?;

        // Update sparse index untuk element yang di-swap
        if dense_idx != last {
            // Cari sparse index yang point ke 'last', update ke dense_idx
            for (i, &opt_idx) in self.sparse.iter().enumerate() {
                if opt_idx == Some(last as u32) {
                    self.sparse[i] = Some(dense_idx as u32);
                    break;
                }
            }
        }

        self.sparse[sparse_idx] = None;
        self.generations[sparse_idx] += 1; // Increment generation
        self.free_list.push(handle.index);

        Some(removed)
    }

    /// Iteration: Dense array = cache-friendly (SOA benefit)
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.dense.iter()
    }
}
```

**Kelebihan:**
- **O(1) insert/remove**: Amortized, tidak seperti Vec::remove
- **Stable handles**: ID bisa disimpan di component lain (graph-friendly)
- **Dense iteration**: Cache-friendly seperti SOA
- **Generational safety**: Handle stale otomatis invalid
- **OOM-proof**: Vec growth tetap bounded oleh system, tapi bisa pre-allocate

---

### 3. Pool Allocator dengan Size Classes

```rust
/// Pool allocator: Fixed-size blocks, no fragmentation
pub struct PoolAllocator<T, const BLOCK_SIZE: usize> {
    blocks: Vec<Box<[T; BLOCK_SIZE]>>, // Pre-allocated blocks
    free_list: Vec<*mut T>,           // Linked list free slots
    used_count: usize,
    max_blocks: usize,                // Hard limit
}

impl<T: Default + Clone, const N: usize> PoolAllocator<T, N> {
    pub fn new(max_blocks: usize) -> Self {
        Self {
            blocks: Vec::with_capacity(max_blocks),
            free_list: Vec::with_capacity(max_blocks * N),
            used_count: 0,
            max_blocks,
        }
    }

    pub fn acquire(&mut self) -> Option<PoolPtr<T>> {
        // Reuse dari free_list
        if let Some(ptr) = self.free_list.pop() {
            self.used_count += 1;
            return Some(PoolPtr { ptr, generation: 0 });
        }

        // Alloc block baru jika masih dalam limit
        if self.blocks.len() >= self.max_blocks {
            return None; // Hard limit reached
        }

        let mut block = Box::new([T::default(); N]);
        let base = block.as_mut_ptr();

        // Push semua slot ke free_list kecuali yang pertama
        for i in (1..N).rev() {
            self.free_list.push(unsafe { base.add(i) });
        }

        self.blocks.push(block);
        self.used_count += 1;

        Some(PoolPtr { ptr: base, generation: 0 })
    }

    pub fn release(&mut self, ptr: PoolPtr<T>) {
        self.free_list.push(ptr.ptr);
        self.used_count -= 1;
    }

    pub fn utilization(&self) -> f32 {
        self.used_count as f32 / (self.blocks.len() * N) as f32
    }
}

pub struct PoolPtr<T> {
    ptr: *mut T,
    generation: u32,
}

impl<T> std::ops::Deref for PoolPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}
```

**Kelebihan:**
- **Zero fragmentation**: Fixed-size blocks
- **Predictable memory**: `max_blocks * BLOCK_SIZE * size_of::<T>()`
- **O(1) acquire/release**: Linked list operation
- **Cache locality**: Blocks contiguous, data dalam block contiguous
- **OOM-proof**: Hard max_blocks, return Option::None jika penuh

---

### 4. Ring Buffer untuk Streaming Data

```rust
/// Ring buffer: Fixed capacity, overwrite oldest (OOM-proof by design)
pub struct RingBuffer<T, const N: usize> {
    buffer: [MaybeUninit<T>; N],
    head: usize, // Write position
    tail: usize, // Read position
    count: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        Self {
            buffer: [MaybeUninit::uninit(); N],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Option<T> {
        let old = if self.count == N {
            // Buffer penuh: overwrite oldest
            let old = unsafe { self.buffer[self.tail].assume_init_read() };
            self.tail = (self.tail + 1) % N;
            Some(old)
        } else {
            self.count += 1;
            None
        };

        self.buffer[self.head].write(value);
        self.head = (self.head + 1) % N;

        old // Return evicted value jika ada
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }

        let value = unsafe { self.buffer[self.tail].assume_init_read() };
        self.tail = (self.tail + 1) % N;
        self.count -= 1;

        Some(value)
    }

    pub fn is_full(&self) -> bool {
        self.count == N
    }
}

// Usage: Log streaming, event buffer, recent history
pub type EventLog = RingBuffer<Event, 10000>; // Max 10K events, OOM-proof
```

**Kelebihan:**
- **Fixed memory**: `N * size_of::<T>()` bytes, never grows
- **OOM-impossible**: Allocation hanya sekali di `new()`
- **Lock-free capable**: Single producer/consumer bisa lock-free
- **Cache-friendly**: Array contiguous, modulo untuk wrap

---

### 5. Comparison: SOA vs Controlled Dynamism

| Pattern | Flexibility | OOM Safety | Cache Efficiency | Use Case |
|---------|-------------|------------|------------------|----------|
| **SOA Statis** | ❌ Fixed | ✅ Hard limit | ✅ Optimal | Physics, rendering |
| **Arena Budget** | ⚠️ Frame-limited | ✅ Soft limit | ✅ Contiguous | Temporary scratch |
| **Sparse Set** | ✅ Dynamic | ⚠️ Vec growth | ✅ Dense iter | Game entities, ECS |
| **Pool Allocator** | ⚠️ Fixed block | ✅ Hard limit | ✅ Block-local | Network packets, jobs |
| **Ring Buffer** | ❌ Overwrite | ✅ Impossible | ✅ Optimal | Streaming, logs |

---

### 6. Hybrid: SOA + Sparse Set untuk ECS Modern

```rust
/// ECS yang menggabungkan SOA untuk data, Sparse Set untuk indexing
pub struct HybridECS {
    // SOA: Data storage (cache-friendly)
    positions: Vec<Vec3>,
    velocities: Vec<Vec3>,
    masses: Vec<f32>,

    // Sparse Set: Entity indexing (fleksibel)
    entity_index: SparseSet<EntityMeta>,

    // Generational handles untuk safety
    generations: Vec<u32>,
}

pub struct Entity {
    index: u32,
    generation: u32,
}

impl HybridECS {
    pub fn spawn(&mut self) -> Option<Entity> {
        // Sparse Set: O(1) insert dengan handle stabil
        let handle = self.entity_index.insert(EntityMeta {
            position_idx: self.positions.len() as u32,
            // ...
        })?;

        // SOA: Push data (amortized growth, bisa pre-allocate)
        self.positions.push(Vec3::default());
        self.velocities.push(Vec3::default());
        self.masses.push(1.0);

        Some(Entity {
            index: handle.index,
            generation: handle.generation,
        })
    }

    pub fn get_position(&self, entity: Entity) -> Option<&Vec3> {
        let meta = self.entity_index.get(Handle {
            index: entity.index,
            generation: entity.generation,
        })?;

        self.positions.get(meta.position_idx as usize)
    }

    /// Iteration: SOA-style (cache-friendly)
    pub fn update_positions(&mut self, dt: f32) {
        for i in 0..self.positions.len() {
            // Linear scan, SIMD-friendly
            self.positions[i] += self.velocities[i] * dt;
        }
    }
}
```

**Kelebihan Hybrid:**
- **SOA benefit**: Cache-friendly iteration, SIMD-ready
- **Sparse Set benefit**: Stable handles, O(1) insert/remove
- **Generational safety**: No dangling pointers
- **OOM control**: Pre-allocate SOA capacity, Sparse Set bounded

---

## 🎯 Decision Tree: Kapan Apa?

```
Need dynamic entity count?
├── NO → SOA Statis (best performance)
│
├── YES → Need stable handles/references?
│   ├── NO → Arena Budget (frame-scoped)
│   │
│   ├── YES → Data mostly iteration-heavy?
│   │   ├── YES → Hybrid ECS (SOA + Sparse Set)
│   │   │
│   │   ├── NO  → Sparse Set (graph, UI)
│   │
│   └── Fixed-size items?
│       ├── YES → Pool Allocator (network, particles)
│       │
│       └── Streaming/overwrite OK?
│           └── YES → Ring Buffer (logs, events)
```

---

## 💡 Key Insight

> **"OOM-proof tidak berarti statis, tapi berarti bounded"**

Semua pattern di atas memberikan **upper bound memori yang predictable**:
- **Arena**: `budget_bytes` hard limit
- **Sparse Set**: `Vec::with_capacity(max)` + handle failure
- **Pool**: `max_blocks` hard limit
- **Ring**: `N` fixed at compile time

Bedanya dengan SOA murni: **fleksibilitas dalam batasan**, bukan **fleksibilitas tanpa batas**.


## 🎯 KESIMPULAN

Buku Hitam ini adalah **default yang aman**, bukan dogma. Rust memungkinkan zero-cost abstraction, tapi abstraction tetap punya cost jika salah pakai.

> *"Premature optimization is the root of all evil. But premature pessimization is the root of all evil too."* — Herb Sutter

Gunakan aturan ini untuk **menghindari pessimization**, bukan untuk micro-optimasi dini. Profile dulu (`cargo flamegraph`, `perf`), lalu apply fix yang relevan.

---

**License: MIT/Apache-2.0**  
**Maintainer: Arsitek RRM**  
**Kontribusi: PR welcome untuk dosa baru**
