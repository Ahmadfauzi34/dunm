use crate::core::config::GLOBAL_DIMENSION;
use fastrand;
use ndarray::{Array1, ArrayView1, ArrayViewMut1};
use std::collections::{HashMap, HashSet};

const MAX_SEEDS: usize = 10000;

/// ============================================
/// LSH INDEX FOR SUBLINEAR SEARCH (O(1))
/// ============================================
struct LshIndex {
    dimension: usize,
    num_projections: usize,
    bucket_count: usize,
    projections: Vec<Array1<f32>>,
    buckets: Vec<HashMap<String, Vec<usize>>>,
}

impl LshIndex {
    fn new(dimension: usize, num_projections: usize, bucket_count: usize) -> Self {
        let mut projections = Vec::with_capacity(num_projections);
        for _ in 0..num_projections {
            let mut proj = Array1::<f32>::zeros(dimension);
            let mut mag_sq: f32 = 0.0;
            for d in 0..dimension {
                let val = fastrand::f32() * 2.0 - 1.0;
                proj[d] = val;
                mag_sq += val * val;
            }

            // L2 Branchless Normalization
            let inv_mag = 1.0 / (mag_sq.sqrt() + 1e-15);
            for d in 0..dimension {
                proj[d] *= inv_mag;
            }
            projections.push(proj);
        }

        let mut buckets = Vec::with_capacity(num_projections);
        for _ in 0..num_projections {
            buckets.push(HashMap::new());
        }

        Self {
            dimension,
            num_projections,
            bucket_count,
            projections,
            buckets,
        }
    }

    fn hash(&self, tensor: &ArrayView1<'_, f32>) -> Vec<String> {
        let mut hashes = Vec::with_capacity(self.num_projections);

        for proj in &self.projections {
            let mut dot = 0.0;
            for d in 0..self.dimension {
                dot += tensor[d] * proj[d];
            }

            // Normalisasi dot product menjadi rentang indeks bucket (0 - bucket_count)
            // Cosine similarity berkisar antara -1.0 dan 1.0
            let normalized = dot.midpoint(1.0);
            let mut bucket_idx = (normalized * self.bucket_count as f32).floor() as usize;

            if bucket_idx >= self.bucket_count {
                bucket_idx = self.bucket_count - 1;
            }

            hashes.push(bucket_idx.to_string());
        }
        hashes
    }

    fn add(&mut self, index: usize, tensor: &ArrayView1<'_, f32>) {
        let hashes = self.hash(tensor);
        for (proj_idx, hash) in hashes.into_iter().enumerate() {
            let bucket = &mut self.buckets[proj_idx];
            bucket.entry(hash).or_default().push(index);
        }
    }

    fn query(&self, tensor: &ArrayView1<'_, f32>, max_candidates: usize) -> Vec<usize> {
        let hashes = self.hash(tensor);
        let mut candidates = HashSet::new();

        for (proj_idx, hash) in hashes.into_iter().enumerate() {
            if let Some(indices) = self.buckets[proj_idx].get(&hash) {
                for &idx in indices {
                    candidates.insert(idx);
                }
            }
        }

        let mut result: Vec<usize> = candidates.into_iter().collect();
        if result.len() > max_candidates {
            result.truncate(max_candidates);
        }
        result
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }
}

/// 🌌 THE LOGIC SEED BANK 🌌
/// Tempat penyimpanan seluruh "Skill" dan "Logika" dalam bentuk Tensor Kontinu.
/// 100% Menggunakan Arsitektur `SoA` (Entity Component System style).
pub struct LogicSeedBank {
    pub active_count: usize,

    pub rule_names: Vec<String>,
    pub rule_seeds: Vec<i32>,
    pub rule_tensors: Vec<f32>,

    lsh_index: LshIndex,
}

impl Default for LogicSeedBank {
    fn default() -> Self {
        Self::new()
    }
}

impl LogicSeedBank {
    pub fn new() -> Self {
        Self {
            active_count: 0,
            rule_names: vec![String::new(); MAX_SEEDS],
            rule_seeds: vec![0; MAX_SEEDS],
            rule_tensors: vec![0.0; MAX_SEEDS * GLOBAL_DIMENSION],
            lsh_index: LshIndex::new(GLOBAL_DIMENSION, 8, 256),
        }
    }

    /// Menambahkan memori ke seed bank
    pub fn add_seed(&mut self, name: &str, seed: i32, tensor: &Array1<f32>) -> Option<usize> {
        if self.active_count >= MAX_SEEDS {
            return None; // Seed bank full
        }

        let idx = self.active_count;
        self.rule_names[idx] = name.to_string();
        self.rule_seeds[idx] = seed;

        let offset = idx * GLOBAL_DIMENSION;
        for d in 0..GLOBAL_DIMENSION {
            self.rule_tensors[offset + d] = tensor[d];
        }

        // To avoid borrow checker conflict, get a view of only the newly inserted data
        let tensor_view = ArrayView1::from(&self.rule_tensors[offset..offset + GLOBAL_DIMENSION]);
        self.lsh_index.add(idx, &tensor_view);

        self.active_count += 1;
        Some(idx)
    }

    /// Helper untuk mengambil sub-array (pointer) ke satu tensor di dalam buffer.
    pub fn get_tensor(&self, index: usize) -> ArrayView1<'_, f32> {
        let offset = index * GLOBAL_DIMENSION;
        ArrayView1::from(&self.rule_tensors[offset..offset + GLOBAL_DIMENSION])
    }

    pub fn get_tensor_mut(&mut self, index: usize) -> ArrayViewMut1<'_, f32> {
        let offset = index * GLOBAL_DIMENSION;
        ArrayViewMut1::from(&mut self.rule_tensors[offset..offset + GLOBAL_DIMENSION])
    }

    /// O(1) query matching rule index using LSH
    pub fn query_similar(&self, target_tensor: &Array1<f32>, max_results: usize) -> Vec<usize> {
        let view = target_tensor.view();
        self.lsh_index.query(&view, max_results)
    }
}
