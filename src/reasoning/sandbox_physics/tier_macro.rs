use crate::core::entity_manifold::EntityManifold;
use crate::core::fhrr::FHRR;
use ndarray::Array1;

pub struct TierMacroPhysics;

impl TierMacroPhysics {
    #[inline(always)]
    pub fn apply_macro_physics(
        u: &mut EntityManifold,
        delta_spatial: &Array1<f32>,
        axiom_type: &str,
    ) -> bool {
        if axiom_type.starts_with("MACRO:") {
            if delta_spatial.iter().any(|&v| v.abs() > 0.0) {
                let sp_mut = &mut u.spatial_tensors;
                let dim = crate::core::config::GLOBAL_DIMENSION;

                for i in 0..u.active_count {
                    let start = i * dim;
                    let end = start + dim;
                    let chunk = ndarray::Array1::from_vec(sp_mut[start..end].to_vec());
                    let new_chunk = FHRR::bind(&chunk, delta_spatial);
                    if let Some(slice) = new_chunk.as_slice() {
                        sp_mut[start..end].copy_from_slice(slice);
                    } else {
                        return false;
                    }
                }
            }
        }
        true
    }
}
