use crate::core::core_seeds::CoreSeeds;
use crate::core::fhrr::FHRR;
use ndarray::Array1;

pub struct UniversalManifold {
    pub r_axis_seed: Array1<f32>,
}

impl Default for UniversalManifold {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalManifold {
    pub fn new() -> Self {
        Self {
            r_axis_seed: FHRR::create(None),
        }
    }

    pub fn encode_coordinate(&self, axis_seed: &Array1<f32>, value: f32) -> Array1<f32> {
        FHRR::fractional_bind(axis_seed, value)
    }

    /// Mengembalikan Tensor Spasial Global dari sebuah Piksel (Posisi absolut di kanvas)
    pub fn build_global_spatial_tensor(&self, rel_x: f32, rel_y: f32) -> Array1<f32> {
        let x_tensor = self.encode_coordinate(CoreSeeds::x_axis_seed(), rel_x);
        let y_tensor = self.encode_coordinate(CoreSeeds::y_axis_seed(), rel_y);
        FHRR::bind(&x_tensor, &y_tensor)
    }

    /// Mengembalikan Tensor Bentuk Lokal (Relatif terhadap pusat massa)
    pub fn build_local_shape_tensor(&self, local_dx: f32, local_dy: f32) -> Array1<f32> {
        let x_tensor = self.encode_coordinate(CoreSeeds::x_axis_seed(), local_dx);
        let y_tensor = self.encode_coordinate(CoreSeeds::y_axis_seed(), local_dy);
        FHRR::bind(&x_tensor, &y_tensor)
    }

    /// Mengembalikan Tensor Semantik (Warna)
    pub fn build_semantic_tensor(&self, token: i32) -> Array1<f32> {
        self.encode_coordinate(CoreSeeds::color_seed(), token as f32)
    }
}
