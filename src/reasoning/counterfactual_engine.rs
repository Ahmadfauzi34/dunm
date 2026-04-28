use crate::core::deterministic_femto::DeterministicFemto;
use crate::core::entity_manifold::EntityManifold;
use crate::reasoning::multiverse_sandbox::MultiverseSandbox;
use crate::reasoning::structures::{Axiom, StructuralSignature};

pub struct CounterfactualEngine {
    pub current_hypothesis: Vec<Axiom>,
    pub simulated_outcome: Option<EntityManifold>,
    pub confidence: f32,
    pub failure_analysis: Option<FailureMode>,
    pub failure_memory: Vec<FailurePattern>,
}

#[derive(Clone, Debug)]
pub enum FailureMode {
    /// Failure as an Energy Landscape Gradient
    /// Instead of boolean binary error, this returns the distance and vector
    /// to the nearest Femto-Well (exact precision target).
    HighEnergyState {
        distance: f32,
        gradient_x: DeterministicFemto, // The direction the tensor should have moved
        gradient_y: DeterministicFemto,
        energy_level: f32, // How bad the mismatch is
    },
}

pub struct FailurePattern {
    pub context_signature: StructuralSignature,
    pub failed_axiom: Axiom,
    pub failure_type: FailureMode,
    pub suggested_correction: Option<Vec<Axiom>>,
}

pub struct SimulationResult {
    pub is_success: bool,
    pub failure: Option<FailureMode>,
    pub final_state: EntityManifold,
}

pub enum SequenceResult {
    Success {
        final_state: EntityManifold,
    },
    Invalid {
        at_step: usize,
        reason: IncompatibilityReason,
    },
    FailedEarly {
        at_step: usize,
        remaining_energy: f32,
    },
}

impl SequenceResult {
    pub fn is_success(&self) -> bool {
        matches!(self, SequenceResult::Success { .. })
    }
}

pub enum IncompatibilityReason {
    StateMismatch,
}

impl Default for CounterfactualEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterfactualEngine {
    pub fn new() -> Self {
        Self {
            current_hypothesis: Vec::new(),
            simulated_outcome: None,
            confidence: 0.0,
            failure_analysis: None,
            failure_memory: Vec::new(),
        }
    }

    pub fn what_if(
        &mut self,
        axiom: &Axiom,
        input: &EntityManifold,
        expected: &EntityManifold,
    ) -> SimulationResult {
        let mut sandbox = input.clone();

        MultiverseSandbox::apply_axiom(
            &mut sandbox,
            &axiom.condition_tensor,
            &axiom.delta_spatial,
            &axiom.delta_semantic,
            axiom.delta_x,
            axiom.delta_y,
            axiom.tier,
            &axiom.name,
        );

        let mut outcome = self.analyze_outcome(&sandbox, expected);
        outcome.final_state = sandbox;

        if let Some(ref failure) = outcome.failure {
            self.learn_from_failure(failure, axiom);
        }

        outcome
    }

    pub fn what_if_sequence(
        &mut self,
        sequence: &[Axiom],
        input: &EntityManifold,
        expected: &EntityManifold,
    ) -> SequenceResult {
        let mut state = input.clone();
        let mut intermediate_results = Vec::new();

        for (i, axiom) in sequence.iter().enumerate() {
            if let Some(_prev_result) = intermediate_results.last() {
                if !self.are_compatible(_prev_result, axiom) {
                    return SequenceResult::Invalid {
                        at_step: i,
                        reason: IncompatibilityReason::StateMismatch,
                    };
                }
            }

            MultiverseSandbox::apply_axiom(
                &mut state,
                &axiom.condition_tensor,
                &axiom.delta_spatial,
                &axiom.delta_semantic,
                axiom.delta_x,
                axiom.delta_y,
                axiom.tier,
                &axiom.name,
            );
            intermediate_results.push(state.clone());

            if self.is_clearly_wrong(&state, expected, i) {
                return SequenceResult::FailedEarly {
                    at_step: i,
                    remaining_energy: self.estimate_remaining_error(&state, expected),
                };
            }
        }

        SequenceResult::Success { final_state: state }
    }

    fn analyze_outcome(
        &self,
        simulated: &EntityManifold,
        expected: &EntityManifold,
    ) -> SimulationResult {
        // Gradient Vector Evaluation
        let mut total_dx = 0.0;
        let mut total_dy = 0.0;
        let mut energy = 0.0;

        let sim_w = simulated.global_width;
        let sim_h = simulated.global_height;
        let exp_w = expected.global_width;
        let exp_h = expected.global_height;

        let mut dim_mismatch = false;
        if sim_w != exp_w || sim_h != exp_h {
            energy += ((sim_w - exp_w).powi(2) + (sim_h - exp_h).powi(2)).sqrt() * 1000.0;
            dim_mismatch = true;
        }

        // Jarak centroid aktual vs target (Mencari vektor arah yang paling tepat untuk perbaikan)
        // Kita menggunakan centroid dari warna yang sama jika memungkinkan
        let count = simulated.active_count;
        let exp_count = expected.active_count;

        let mut matched_entities = 0;

        for i in 0..count {
            let sim_t = simulated.tokens[i];
            let sim_x = simulated.centers_x[i];
            let sim_y = simulated.centers_y[i];

            let mut min_dist = 99999.0;
            let mut closest_dx = 0.0;
            let mut closest_dy = 0.0;

            for j in 0..exp_count {
                if expected.tokens[j] == sim_t {
                    let exp_x = expected.centers_x[j];
                    let exp_y = expected.centers_y[j];

                    // Gradient is Target - Current = Direction to move
                    let dx = exp_x - sim_x;
                    let dy = exp_y - sim_y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < min_dist {
                        min_dist = dist;
                        closest_dx = dx;
                        closest_dy = dy;
                    }
                }
            }

            if min_dist < 9999.0 {
                total_dx += closest_dx;
                total_dy += closest_dy;
                energy += min_dist;
                matched_entities += 1;
            } else {
                energy += 50.0; // Penalty untuk objek yang tak punya pasangan
            }
        }

        if dim_mismatch || energy > 0.1 || matched_entities != exp_count {
            // Evaluasi vektor gradien optimal untuk mental replay:
            // Rata-rata vektor geseran hanya efektif jika obyeknya satu.
            // Pada task multiobyek yang bergerak identik, rata-rata adalah representasi pergerakan global.
            let mut avg_dx = DeterministicFemto::ZERO;
            let mut avg_dy = DeterministicFemto::ZERO;

            if matched_entities > 0 {
                // Konversi total akumulasi menjadi fixed-point DULU, baru dibagi secara integer (deterministic)
                let total_dx_fixed = DeterministicFemto::from_f32(total_dx);
                let total_dy_fixed = DeterministicFemto::from_f32(total_dy);
                avg_dx = total_dx_fixed / (matched_entities as i64);
                avg_dy = total_dy_fixed / (matched_entities as i64);
            }

            let avg_dx_f32 = avg_dx.to_f32();
            let avg_dy_f32 = avg_dy.to_f32();
            let dist = (avg_dx_f32 * avg_dx_f32 + avg_dy_f32 * avg_dy_f32).sqrt();

            return SimulationResult {
                is_success: false,
                failure: Some(FailureMode::HighEnergyState {
                    distance: dist,
                    gradient_x: avg_dx,
                    gradient_y: avg_dy,
                    energy_level: energy,
                }),
                final_state: simulated.clone(),
            };
        }

        SimulationResult {
            is_success: true,
            failure: None,
            final_state: simulated.clone(),
        }
    }

    fn learn_from_failure(&mut self, failure: &FailureMode, attempted_axiom: &Axiom) {
        let pattern = FailurePattern {
            context_signature: self.extract_signature(),
            failed_axiom: attempted_axiom.clone(),
            failure_type: failure.clone(),
            suggested_correction: self.suggest_correction(failure),
        };
        self.failure_memory.push(pattern);
    }

    fn extract_signature(&self) -> StructuralSignature {
        use crate::reasoning::structures::{DimensionRelation, ObjectDelta, TopologyHint};
        StructuralSignature {
            dim_relation: DimensionRelation::Equal,
            object_delta: ObjectDelta::Same,
            color_mapping: None,
            topology_hint: TopologyHint::Grid,
        }
    }

    pub fn suggest_correction(&self, failure: &FailureMode) -> Option<Vec<Axiom>> {
        match failure {
            FailureMode::HighEnergyState {
                gradient_x,
                gradient_y,
                ..
            } => {
                // Konversi vektor kegagalan menjadi tebakan solusi dengan geometri Tensor FHRR
                // Kita mentranslasikan "distance-to-well" ke dalam bentuk Fractional Bind
                if gradient_x.abs() > DeterministicFemto::ZERO
                    || gradient_y.abs() > DeterministicFemto::ZERO
                {
                    let rx = gradient_x.to_f32().round();
                    let ry = gradient_y.to_f32().round();

                    let x_seed = crate::core::core_seeds::CoreSeeds::x_axis_seed();
                    let y_seed = crate::core::core_seeds::CoreSeeds::y_axis_seed();
                    let tensor_spatial =
                        crate::core::fhrr::FHRR::fractional_bind_2d(x_seed, rx, y_seed, ry);

                    let mut correction = Axiom::identity();
                    correction.name = format!("SUGGESTED_TRANS_{}_{}", rx, ry);
                    correction.delta_x = rx;
                    correction.delta_y = ry;
                    correction.delta_spatial = tensor_spatial;
                    correction.tier = 3; // Relational / Spatial Move Tier
                    Some(vec![correction])
                } else {
                    // Jika dimensinya hancur atau gradient 0.0 tapi energi tinggi,
                    // kita coba CROP_TO_COLOR atau SPAWN
                    Some(vec![Axiom::crop_to_content()])
                }
            }
        }
    }

    fn are_compatible(&self, _prev: &EntityManifold, _next: &Axiom) -> bool {
        true
    }

    fn is_clearly_wrong(
        &self,
        _state: &EntityManifold,
        _expected: &EntityManifold,
        _step: usize,
    ) -> bool {
        false
    }

    fn estimate_remaining_error(&self, _state: &EntityManifold, _expected: &EntityManifold) -> f32 {
        100.0
    }
}
