use crate::core::entity_manifold::EntityManifold;

pub struct CognitiveLoop;

impl CognitiveLoop {
    pub fn calculate_dark_matter(manifold: &EntityManifold) -> f32 {
        if manifold.active_count == 0 {
            return 0.0;
        }
        let mut zero_mass_count = 0;
        for i in 0..manifold.active_count {
            if manifold.masses[i] == 0.0 {
                zero_mass_count += 1;
            }
        }
        zero_mass_count as f32 / manifold.active_count as f32
    }
}
