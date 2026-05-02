use crate::core::entity_manifold::EntityManifold;
use crate::reasoning::counterfactual_engine::CounterfactualEngine;
use crate::reasoning::structures::{Axiom, StructuralSignature};

pub struct CausalReasoner {
    engine: CounterfactualEngine,
}

pub struct CausalAssessment {
    pub intervention: Axiom,
    pub is_necessary: bool,
    pub is_sufficient: bool,
    pub is_specific: bool,
    pub confidence: f32,
    pub explanation: String,
}

impl Default for CausalReasoner {
    fn default() -> Self {
        Self::new()
    }
}

impl CausalReasoner {
    pub fn new() -> Self {
        Self {
            engine: CounterfactualEngine::new(),
        }
    }

    pub fn assess_causality(
        &mut self,
        intervention: &Axiom,
        initial: &EntityManifold,
        expected_effect: &StructuralSignature,
        alternatives: &[Axiom],
    ) -> CausalAssessment {
        // 1. Uji intervensi aktual (Aksi aktual)
        let actual = self.engine.what_if(intervention, initial, initial);
        let actual_sig = self.extract_signature(initial, &actual.final_state);

        // 2. Uji counterfactual (Tidak melakukan apa-apa)
        let identity = Axiom::identity();
        let counterfactual = self.engine.what_if(&identity, initial, initial);
        let counter_sig = self.extract_signature(initial, &counterfactual.final_state);

        // 3. Uji alternatif (Apakah aksi lain bisa menghasilkan efek yang sama?)
        let alt_results: Vec<_> = alternatives
            .iter()
            .map(|alt| self.engine.what_if(alt, initial, initial))
            .collect();

        // Logika Kausalitas Murni
        let necessary = counter_sig != *expected_effect; // Jika tidak dilakukan, efek TIDAK terjadi
        let sufficient = actual_sig == *expected_effect; // Jika dilakukan, efek TERJADI

        let mut specific = true;
        for alt_res in alt_results {
            let alt_sig = self.extract_signature(initial, &alt_res.final_state);
            if alt_sig == *expected_effect {
                specific = false; // Aksi lain ternyata juga bisa! (TIDAK SPESIFIK)
                break;
            }
        }

        let mut confidence = 0.5;
        if necessary && sufficient {
            confidence += 0.3;
        }
        if specific {
            confidence += 0.2;
        }

        let explanation = format!(
            "[{}] -> Necessary: {}, Sufficient: {}, Specific: {} | Conf: {:.2}",
            intervention.name, necessary, sufficient, specific, confidence
        );

        CausalAssessment {
            intervention: intervention.clone(),
            is_necessary: necessary,
            is_sufficient: sufficient,
            is_specific: specific,
            confidence,
            explanation,
        }
    }

    pub fn extract_signature(
        &self,
        initial: &EntityManifold,
        final_state: &EntityManifold,
    ) -> StructuralSignature {
        use crate::reasoning::structures::{DimensionRelation, ObjectDelta, TopologyHint};

        let dim_relation = if final_state.global_width > initial.global_width {
            DimensionRelation::Larger
        } else if final_state.global_width < initial.global_width {
            DimensionRelation::Smaller
        } else {
            DimensionRelation::Equal
        };

        let object_delta = match final_state.active_count.cmp(&initial.active_count) {
            std::cmp::Ordering::Greater => ObjectDelta::Added,
            std::cmp::Ordering::Less => ObjectDelta::Removed,
            std::cmp::Ordering::Equal => ObjectDelta::Same, // Simplification
        };

        StructuralSignature {
            dim_relation,
            object_delta,
            color_mapping: None,
            topology_hint: TopologyHint::Grid,
        }
    }
}
