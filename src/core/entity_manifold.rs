use crate::core::config::GLOBAL_DIMENSION;
use crate::core::cow_memory::SmartMultiverseState;
use ndarray::{Array1, ArrayViewMut1};

/// Struktur `SoA` (Structure of Arrays) untuk Quantum Entity Manifold.
/// Didesain untuk Zero-GC dan cache locality di L1/L2 secara dinamis.
/// Menggunakan sistem Tri-Tensor: Spatial (Pusat Global), Shape (Pola Lokal), dan Semantic (Warna).
pub struct EntityManifold {
    pub active_count: usize,
    pub global_width: f32,
    pub global_height: f32,

    // 1. Posisi Global (Pusat Massa Absolut di Kanvas)
    pub spatial_tensors: Vec<f32>,

    // 2. Cetak Biru (Blueprint) Relatif (Bentuk lokal dari titik pusat)
    pub shape_tensors: Vec<f32>,

    // 3. Warna / Tipe Material
    pub semantic_tensors: Vec<f32>,

    pub ids: Vec<String>,
    pub masses: Vec<f32>,
    pub tokens: Vec<i32>,

    // Spans / Bounding Boxes Anisotropik
    pub spans_x: Vec<f32>,
    pub spans_y: Vec<f32>,

    // Dense CoW representation for O(1) cloning grid operations
    pub cow_grid: Option<SmartMultiverseState>,

    // Pusat Massa Kinetik / Scalar Momentum
    pub centers_x: Vec<f32>,
    pub centers_y: Vec<f32>,
    pub momentums_x: Vec<f32>,
    pub momentums_y: Vec<f32>,

    // Status Jeratan (Entanglement)
    pub entanglement_status: Vec<f32>,
}

impl Default for EntityManifold {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EntityManifold {
    /// Custom Clone to avoid copying unused pre-allocated capacity (reduces MCTS cache thrashing).
    fn clone(&self) -> Self {
        let count = self.active_count;
        let t_count = count * GLOBAL_DIMENSION;

        let mut new_manifold = Self {
            active_count: count,
            global_width: self.global_width,
            global_height: self.global_height,
            masses: Vec::with_capacity(count),
            tokens: Vec::with_capacity(count),
            spans_x: Vec::with_capacity(count),
            spans_y: Vec::with_capacity(count),
            centers_x: Vec::with_capacity(count),
            centers_y: Vec::with_capacity(count),
            momentums_x: Vec::with_capacity(count),
            momentums_y: Vec::with_capacity(count),
            entanglement_status: Vec::with_capacity(count),
            spatial_tensors: Vec::with_capacity(t_count),
            shape_tensors: Vec::with_capacity(t_count),
            semantic_tensors: Vec::with_capacity(t_count),
            ids: Vec::with_capacity(count),
            cow_grid: self.cow_grid.clone(),
        };

        if count > 0 {
            new_manifold.masses.extend_from_slice(&self.masses[..count]);
            new_manifold.tokens.extend_from_slice(&self.tokens[..count]);
            new_manifold.spans_x.extend_from_slice(&self.spans_x[..count]);
            new_manifold.spans_y.extend_from_slice(&self.spans_y[..count]);
            new_manifold.centers_x.extend_from_slice(&self.centers_x[..count]);
            new_manifold.centers_y.extend_from_slice(&self.centers_y[..count]);
            new_manifold.momentums_x.extend_from_slice(&self.momentums_x[..count]);
            new_manifold.momentums_y.extend_from_slice(&self.momentums_y[..count]);
            new_manifold.entanglement_status.extend_from_slice(&self.entanglement_status[..count]);

            new_manifold.spatial_tensors.extend_from_slice(&self.spatial_tensors[..t_count]);
            new_manifold.shape_tensors.extend_from_slice(&self.shape_tensors[..t_count]);
            new_manifold.semantic_tensors.extend_from_slice(&self.semantic_tensors[..t_count]);

            // IDs are Strings, so we map and clone just the active ones
            new_manifold.ids.extend(self.ids.iter().take(count).cloned());
        }

        new_manifold
    }
}

impl EntityManifold {
    pub fn new() -> Self {
        Self {
            active_count: 0,
            global_width: 0.0,
            global_height: 0.0,
            spatial_tensors: Vec::new(),
            shape_tensors: Vec::new(),
            semantic_tensors: Vec::new(),
            ids: Vec::new(),
            masses: Vec::new(),
            tokens: Vec::new(),
            spans_x: Vec::new(),
            spans_y: Vec::new(),
            centers_x: Vec::new(),
            centers_y: Vec::new(),
            cow_grid: None,
            entanglement_status: Vec::new(),
            momentums_x: Vec::new(),
            momentums_y: Vec::new(),
        }
    }

    /// Dynamic capacity extension for dense array mapping (`SparseSet` behavior fallback)
    pub fn ensure_scalar_capacity(&mut self, required_len: usize) {
        if self.masses.len() < required_len {
            let add = required_len - self.masses.len();
            self.ids.extend(std::iter::repeat_n(String::new(), add));
            self.masses.extend(std::iter::repeat_n(0.0, add));
            self.tokens.extend(std::iter::repeat_n(0, add));
            self.spans_x.extend(std::iter::repeat_n(0.0, add));
            self.spans_y.extend(std::iter::repeat_n(0.0, add));
            self.centers_x.extend(std::iter::repeat_n(0.0, add));
            self.centers_y.extend(std::iter::repeat_n(0.0, add));
            self.entanglement_status
                .extend(std::iter::repeat_n(0.0, add));
            self.momentums_x.extend(std::iter::repeat_n(0.0, add));
            self.momentums_y.extend(std::iter::repeat_n(0.0, add));
        }
    }

    /// Fungsi bantuan agar `Vec` tensor tetap cukup ukurannya saat index diakses
    pub fn ensure_tensor_capacity(&mut self, required_len: usize) {
        if self.spatial_tensors.len() < required_len {
            self.spatial_tensors.resize(required_len, 0.0);
            self.shape_tensors.resize(required_len, 0.0);
            self.semantic_tensors.resize(required_len, 0.0);
        }
    }

    pub fn get_spatial_tensor_mut(&mut self, index: usize) -> ArrayViewMut1<'_, f32> {
        let offset = index * GLOBAL_DIMENSION;
        let required = offset + GLOBAL_DIMENSION;
        self.ensure_tensor_capacity(required);
        ArrayViewMut1::from(&mut self.spatial_tensors[offset..required])
    }

    pub fn get_spatial_tensor(&self, index: usize) -> Array1<f32> {
        let offset = index * GLOBAL_DIMENSION;
        let required = offset + GLOBAL_DIMENSION;
        if self.spatial_tensors.len() >= required {
            Array1::from_vec(self.spatial_tensors[offset..required].to_vec())
        } else {
            Array1::zeros(GLOBAL_DIMENSION)
        }
    }

    pub fn get_shape_tensor_mut(&mut self, index: usize) -> ArrayViewMut1<'_, f32> {
        let offset = index * GLOBAL_DIMENSION;
        let required = offset + GLOBAL_DIMENSION;
        self.ensure_tensor_capacity(required);
        ArrayViewMut1::from(&mut self.shape_tensors[offset..required])
    }

    pub fn get_shape_tensor(&self, index: usize) -> Array1<f32> {
        let offset = index * GLOBAL_DIMENSION;
        let required = offset + GLOBAL_DIMENSION;
        if self.shape_tensors.len() >= required {
            Array1::from_vec(self.shape_tensors[offset..required].to_vec())
        } else {
            Array1::zeros(GLOBAL_DIMENSION)
        }
    }

    pub fn get_semantic_tensor_mut(&mut self, index: usize) -> ArrayViewMut1<'_, f32> {
        let offset = index * GLOBAL_DIMENSION;
        let required = offset + GLOBAL_DIMENSION;
        self.ensure_tensor_capacity(required);
        ArrayViewMut1::from(&mut self.semantic_tensors[offset..required])
    }

    pub fn get_semantic_tensor(&self, index: usize) -> Array1<f32> {
        let offset = index * GLOBAL_DIMENSION;
        let required = offset + GLOBAL_DIMENSION;
        if self.semantic_tensors.len() >= required {
            Array1::from_vec(self.semantic_tensors[offset..required].to_vec())
        } else {
            Array1::zeros(GLOBAL_DIMENSION)
        }
    }
}

impl EntityManifold {
    /// Sinkronisasi Sparse SOA -> Dense `CoW` Grid.
    /// Fungsi ini mengubah token-token warna 1D dari `EntityManifold` menjadi
    /// representasi dense `SmartMultiverseState` 2D.
    pub fn sync_to_cow(&mut self) {
        if self.global_width <= 0.0 || self.global_height <= 0.0 {
            return;
        }

        let w = self.global_width as usize;
        let h = self.global_height as usize;

        // Tentukan jumlah chunk
        // Misalnya CHUNK_SIZE = 64
        let width_chunks = w.div_ceil(crate::core::cow_memory::CHUNK_SIZE);
        let height_chunks = h.div_ceil(crate::core::cow_memory::CHUNK_SIZE);

        let mut needs_new_grid = false;
        if let Some(ref grid) = self.cow_grid {
            if grid.width_chunks != width_chunks || grid.height_chunks != height_chunks {
                needs_new_grid = true;
            }
        } else {
            needs_new_grid = true;
        }

        if needs_new_grid {
            self.cow_grid = Some(SmartMultiverseState::new(width_chunks, height_chunks));
        }

        // Sekarang sinkronisasi data warna ke dalam grid
        // Kita set grid ke 0 terlebih dahulu (blank state) -
        // Ini mungkin agak mahal jika kita set 0 secara manual tiap kali sync,
        // tapi dalam praktiknya, cow_grid idealnya sudah menjadi the source of truth suatu saat nanti.
        // Untuk "cicilan", kita akan asumsikan kita replace dengan grid baru atau bersihkan.

        if let Some(ref mut grid) = self.cow_grid {
            // Warning: Jika kita bersihkan seluruh grid lama, kita merusak Copy-on-Write sharing
            // dengan mutasi massal. Idealnya, sync ini cuma dipanggil SEKALI saat inisialisasi awal.
            // Setelah MCTS berjalan, MCTS cukup mengubah grid via modify_cell.
            for i in 0..self.active_count {
                if self.masses[i] > 0.0 {
                    let x = self.centers_x[i].round() as i32;
                    let y = self.centers_y[i].round() as i32;
                    if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                        grid.modify_cell(x as usize, y as usize, self.tokens[i] as f32);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_manifold_new() {
        let manifold = EntityManifold::new();

        assert_eq!(manifold.active_count, 0);
        assert_eq!(manifold.global_width, 0.0);
        assert_eq!(manifold.global_height, 0.0);

        assert!(manifold.spatial_tensors.is_empty());
        assert!(manifold.shape_tensors.is_empty());
        assert!(manifold.semantic_tensors.is_empty());

        assert!(manifold.ids.is_empty());
        assert!(manifold.masses.is_empty());
        assert!(manifold.tokens.is_empty());

        assert!(manifold.spans_x.is_empty());
        assert!(manifold.spans_y.is_empty());

        assert!(manifold.centers_x.is_empty());
        assert!(manifold.centers_y.is_empty());
        assert!(manifold.momentums_x.is_empty());
        assert!(manifold.momentums_y.is_empty());

        assert!(manifold.entanglement_status.is_empty());

        assert!(manifold.cow_grid.is_none());
    }

    #[test]
    fn test_entity_manifold_default() {
        let manifold = EntityManifold::default();

        assert_eq!(manifold.active_count, 0);
        assert_eq!(manifold.global_width, 0.0);
        assert_eq!(manifold.global_height, 0.0);

        assert!(manifold.spatial_tensors.is_empty());
        assert!(manifold.shape_tensors.is_empty());
        assert!(manifold.semantic_tensors.is_empty());

        assert!(manifold.ids.is_empty());
        assert!(manifold.masses.is_empty());
        assert!(manifold.tokens.is_empty());

        assert!(manifold.spans_x.is_empty());
        assert!(manifold.spans_y.is_empty());

        assert!(manifold.centers_x.is_empty());
        assert!(manifold.centers_y.is_empty());
        assert!(manifold.momentums_x.is_empty());
        assert!(manifold.momentums_y.is_empty());

        assert!(manifold.entanglement_status.is_empty());

        assert!(manifold.cow_grid.is_none());
    }
}
