use ndarray::Array1;
use std::sync::Arc;

/// =============================================================================
/// CONTINUUM FIELD - Single Tensor Landscape
/// Menghancurkan sekat hierarki Makro/Meso/Mikro. Seluruh entitas eksis di dalam
/// satu ruang f32 kontinu tunggal yang dipandu oleh toleransi (Energy Well Width).
/// =============================================================================
#[derive(Clone)]
pub struct ContinuumField {
    pub entities: Arc<Vec<ContinuousEntity>>,
    pub complexity_map: Vec<f32>,
    pub precision_width: f64, // Toleransi femto ke micro (1e-15 -> 1e-6)
}

#[derive(Clone)]
pub struct ContinuousEntity {
    pub id: u32,
    pub center_x: f32,
    pub center_y: f32,
    pub color_signature: i32,
    pub spatial_tensor: Array1<f32>,
    pub semantic_tensor: Array1<f32>,
    pub mass: f32,
}

impl ContinuumField {
    pub fn new(capacity: usize) -> Self {
        Self {
            entities: Arc::new(Vec::with_capacity(capacity)),
            complexity_map: Vec::new(),
            precision_width: 1e-6, // Start with Micro precision (fuzzy)
        }
    }

    /// Set ketajaman energi (Adaptive Funneling)
    pub fn set_precision(&mut self, tolerance: f64) {
        self.precision_width = tolerance;
    }
}

// Untuk backwards compatibility (menghindari merombak seluruh dependensi di luar scope PR)
// Kita buat alias sementara untuk CoarseData
#[derive(Clone)]
pub struct CoarseData {
    pub signatures: Arc<Vec<Array1<f32>>>,
    // Fake backward mapping
    pub regions: Arc<Vec<MacroRegion>>,
}

#[derive(Clone)]
pub struct MacroRegion {
    pub bounds: (f32, f32, f32, f32),
}

pub type InfiniteDetailField = ContinuumField;
