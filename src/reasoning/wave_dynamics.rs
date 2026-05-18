use crate::core::config::GLOBAL_DIMENSION;
use crate::core::entity_manifold::EntityManifold;
use crate::reasoning::entanglement_graph::EntanglementGraph;
use crate::reasoning::entanglement_optimizer::EntanglementOptimizer;
use ndarray::Array1;

/// 🌊 WAVE DYNAMICS (Fase 4)
/// OOP-Free Hebbian Learning Matrix & Huygens-Fresnel Navigation.
pub struct WaveDynamics {
    // 🏛️ HUKUM 8: Gunakan satu blok memori (N * N) agar L1 Cache tidak meleset
    pub entanglement_graph: EntanglementGraph,
}

impl Default for WaveDynamics {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveDynamics {
    pub fn new() -> Self {
        Self {
            entanglement_graph: EntanglementGraph::new(),
        }
    }

    /// Mendaftarkan active boundaries. Unlike TS which holds a ref, we pass manifold explicitly.
    pub fn initialize_entities(&mut self, manifold: &EntityManifold) {
        let count = manifold.active_count;
        self.entanglement_graph.reset_active(count);
    }

    /// Menjalankan Hebbian Learning di atas Structure of Arrays.
    pub fn evolve_entanglement(&mut self, manifold: &EntityManifold, learning_rate: f32) {
        EntanglementOptimizer::optimize(manifold, &mut self.entanglement_graph, learning_rate);
    }

    /// Memperbarui tensor inPlace berdasarkan daya pikat/tolak (Contrastive Update).
    fn contrastive_update_in_place(
        agent_tensor: &mut Array1<f32>,
        repulsor_tensor: &Array1<f32>,
        attractor_tensor: &Array1<f32>,
        alpha: f32,
    ) {
        // Tarik ke arah attractor, tolak dari repulsor
        for i in 0..GLOBAL_DIMENSION {
            agent_tensor[i] += alpha * (attractor_tensor[i] - repulsor_tensor[i]);
        }

        // L2 Normalization
        let mut mag_sq = 0.0;
        for i in 0..GLOBAL_DIMENSION {
            mag_sq += agent_tensor[i] * agent_tensor[i];
        }
        let mag = mag_sq.sqrt();

        // Menggunakan EPSILON agar tidak perlu if (mag > 0)
        let inv_mag = 1.0 / (mag + 1e-15);
        for i in 0..GLOBAL_DIMENSION {
            agent_tensor[i] *= inv_mag;
        }
    }

    /// WAVE GRAVITY DRIVE (Huygens-Fresnel Navigation)
    /// Mengkalkulasi pergerakan agen berdasarkan atraktor dan repulsor tensor di sekitarnya.
    pub fn apply_wave_gravity(
        &self,
        manifold: &mut EntityManifold,
        agent_index: usize,
        attractors: &[&Array1<f32>],
        repulsors: &[&Array1<f32>],
    ) {
        let mut total_attractor = Array1::<f32>::zeros(GLOBAL_DIMENSION);
        for attr in attractors {
            for i in 0..GLOBAL_DIMENSION {
                total_attractor[i] += attr[i];
            }
        }

        let mut total_repulsor = Array1::<f32>::zeros(GLOBAL_DIMENSION);
        for rep in repulsors {
            for i in 0..GLOBAL_DIMENSION {
                total_repulsor[i] += rep[i];
            }
        }

        let mut agent_tensor = manifold.get_spatial_tensor(agent_index);
        Self::contrastive_update_in_place(
            &mut agent_tensor,
            &total_repulsor,
            &total_attractor,
            0.8,
        );

        let mut t_mut = manifold.get_spatial_tensor_mut(agent_index);
        for i in 0..GLOBAL_DIMENSION {
            t_mut[i] = agent_tensor[i];
        }
    }

    /// TRIGGER COLLAPSE
    /// Meruntuhkan gelombang informasi ke agen-agen yang saling terikat (Entangled).
    pub fn trigger_collapse(&self, manifold: &mut EntityManifold, source_index: usize) {
        let num_entities = manifold.active_count;

        let source_tensor = manifold.get_spatial_tensor(source_index).to_owned();

        // Menggunakan Custom Iterator Zero-Cost dari CSR Graph
        for (target_index, entanglement_weight) in self.entanglement_graph.iter_row(source_index) {
            if target_index >= num_entities
                || target_index == source_index
                || manifold.masses[target_index] == 0.0
            {
                continue;
            }

            {
                let mut target_tensor = manifold.get_spatial_tensor_mut(target_index);
                for i in 0..GLOBAL_DIMENSION {
                    target_tensor[i] = (entanglement_weight * source_tensor[i])
                        + ((1.0 - entanglement_weight) * target_tensor[i]);
                }
            }

            let current_status = manifold.entanglement_status[target_index];
            if entanglement_weight > current_status {
                manifold.entanglement_status[target_index] = entanglement_weight;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity_manifold::EntityManifold;

    #[test]
    fn test_initialize_entities_zero_count() {
        let mut wave_dynamics = WaveDynamics::new();
        let mut manifold = EntityManifold::new();
        manifold.active_count = 0;

        wave_dynamics.initialize_entities(&manifold);

        assert_eq!(wave_dynamics.entanglement_graph.row_ptr.len(), 1);
        assert_eq!(wave_dynamics.entanglement_graph.values.len(), 0);
        assert_eq!(wave_dynamics.entanglement_graph.col_indices.len(), 0);
    }

    #[test]
    fn test_initialize_entities_non_zero_count() {
        let mut wave_dynamics = WaveDynamics::new();
        let mut manifold = EntityManifold::new();
        manifold.active_count = 5;

        wave_dynamics.initialize_entities(&manifold);

        assert_eq!(wave_dynamics.entanglement_graph.row_ptr.len(), 6);
        assert_eq!(wave_dynamics.entanglement_graph.values.len(), 5);
        assert_eq!(wave_dynamics.entanglement_graph.col_indices.len(), 5);

        // Verify self-loops
        for i in 0..5 {
            assert_eq!(wave_dynamics.entanglement_graph.row_ptr[i], i);
            assert_eq!(wave_dynamics.entanglement_graph.col_indices[i], i);
            assert_eq!(wave_dynamics.entanglement_graph.values[i], 1.0);
        }
        assert_eq!(wave_dynamics.entanglement_graph.row_ptr[5], 5);
    }
}
