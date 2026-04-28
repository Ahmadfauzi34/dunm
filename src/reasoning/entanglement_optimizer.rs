use crate::core::entity_manifold::EntityManifold;
use crate::core::fhrr::FHRR;
use crate::reasoning::entanglement_graph::EntanglementGraph;

pub struct EntanglementOptimizer;

impl EntanglementOptimizer {
    /// 🕸️ HEBBIAN ENTANGLEMENT UPDATE
    /// "Neurons that fire together, wire together."
    pub fn optimize(manifold: &EntityManifold, graph: &mut EntanglementGraph, learning_rate: f32) {
        let num_entities = manifold.active_count;

        let mut new_graph = EntanglementGraph {
            values: Vec::with_capacity(num_entities * 10), // Heuristic sparsity
            col_indices: Vec::with_capacity(num_entities * 10),
            row_ptr: vec![0; num_entities + 1],
        };

        for i in 0..num_entities {
            new_graph.row_ptr[i] = new_graph.values.len();

            if manifold.masses[i] == 0.0 {
                continue;
            }

            let tensor_a = manifold.get_spatial_tensor(i);

            // Batasan Spasial / Radius Lokal (Contoh Filter Heuristik)
            // Hanya evaluasi agen yang mungkin ter-entangle untuk menghindari loop N^2
            let cx_a = manifold.centers_x[i];
            let cy_a = manifold.centers_y[i];

            for j in 0..num_entities {
                if manifold.masses[j] == 0.0 {
                    continue;
                }

                // Gunakan Filter Spasial: Hanya agen dengan jarak spasial terdekat
                let cx_b = manifold.centers_x[j];
                let cy_b = manifold.centers_y[j];
                let dist_sq = (cx_a - cx_b) * (cx_a - cx_b) + (cy_a - cy_b) * (cy_a - cy_b);

                // Radius toleransi entanglement (Misal: 50.0 radius)
                if dist_sq > 2500.0 && i != j {
                    continue;
                }

                let tensor_b = manifold.get_spatial_tensor(j);
                let coherence = FHRR::similarity(&tensor_a, &tensor_b);

                // Get previous weight (0.0 if not found)
                let current_e = graph.get_weight_csr(i, j);
                let new_e = (current_e + (coherence * learning_rate)).clamp(0.0, 1.0);

                if new_e > 0.001 {
                    // Sparsity Threshold
                    new_graph.values.push(new_e);
                    new_graph.col_indices.push(j);
                }
            }
        }

        let total_vals = new_graph.values.len();
        new_graph.row_ptr[num_entities] = total_vals;

        *graph = new_graph;
    }
}
