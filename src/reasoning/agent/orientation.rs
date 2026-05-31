use crate::core::entity_manifold::EntityManifold;
use crate::perception::structural_analyzer::StructuralAnalyzer;
use crate::perception::structural_analyzer::StructuralDelta;
use crate::self_awareness::self_reflection::SelfReflection;

pub struct OrientationEngine;

impl OrientationEngine {
    pub fn assess_pre_emptive_intent(
        train_states: &[(EntityManifold, EntityManifold)],
        self_reflection: &mut SelfReflection,
    ) -> (Option<StructuralDelta>, usize) {
        println!("🧠 [Orientasi Pre-emptive] Membaca Niat Task...");
        let mut pre_emptive_delta = None;
        let mut betti_1_holes = 0;

        if let Some((man_in, man_out)) = train_states.first() {
            let delta = StructuralAnalyzer::analyze(man_in, man_out);
            let report = self_reflection.assess_situation(&delta);
            println!(
                "   -> Niat / Klasifikasi Masalah: {}",
                report.situation_assessment
            );
            pre_emptive_delta = Some(delta);

            let qcc = crate::quantum_topology::QuantumCellComplex::from_manifold(man_in, 1.5);
            betti_1_holes = *qcc.betti_numbers.get(1).unwrap_or(&0);

            if betti_1_holes > 0 {
                println!(
                    "🧠 [Topologi Kuantum] Betti-1: Mendeteksi {} lubang (holes).",
                    betti_1_holes
                );
            } else {
                println!(
                    "🧠 [Topologi Kuantum] Betti-1: Tidak ada lubang terdeteksi. Struktur solid."
                );
            }
        }

        (pre_emptive_delta, betti_1_holes)
    }

    pub fn calculate_saliency_ratio(
        train_states: &[(EntityManifold, EntityManifold)],
        self_reflection: &mut SelfReflection,
    ) {
        if let Some((man_in, _)) = train_states.first() {
            let mut total_active_mass = 0.0;
            for i in 0..man_in.active_count {
                if man_in.tokens[i] != 0 {
                    total_active_mass += man_in.masses[i];
                }
            }
            let total_area = (man_in.global_width * man_in.global_height).max(1.0);
            self_reflection.active_saliency_ratio = total_active_mass / total_area;
        }
    }
}
