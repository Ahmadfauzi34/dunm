use crate::core::entity_manifold::EntityManifold;
use crate::reasoning::multiverse_sandbox::MultiverseSandbox;
use crate::reasoning::quantum_search_simd::{CognitivePhase, SimdEnergyCalculator};
use ndarray::Array1;

#[derive(Clone, PartialEq)]
pub enum SimulationMode {
    StrictVSA,      // Pragmatic only (Pixel-perfect)
    Probabilistic,  // High Epistemic (Curiosity-driven, toleransi noise)
    Counterfactual, // Horizon planning (What-If scenarios)
}

#[derive(Clone)]
pub struct PolicyAxiom {
    pub condition_tensor: Option<Array1<f32>>,
    pub tensor_rule: Array1<f32>,
    pub delta_x: f32,
    pub delta_y: f32,
    pub physics_tier: u8,
    pub description: String,
}

#[derive(Clone)]
pub struct Policy {
    pub actions: Vec<PolicyAxiom>, // Urutan aksioma (Horizon)
    pub horizon: usize,
}

pub struct DeepActiveInferenceEngine {
    pub current_mode: SimulationMode,
}

impl Default for DeepActiveInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DeepActiveInferenceEngine {
    pub fn new() -> Self {
        Self {
            current_mode: SimulationMode::StrictVSA,
        }
    }

    pub fn switch_mode(&mut self, mode: SimulationMode) {
        self.current_mode = mode;
    }

    /// Evaluasi G(π) = Pragmatic Error - Epistemic Value
    pub fn calculate_expected_free_energy(
        &self,
        initial_state: &EntityManifold,
        expected_grid: &[Vec<i32>],
        policy: &Policy,
    ) -> f32 {
        let mut temp_state = initial_state.clone();
        let mut total_pragmatic = 0.0;
        let mut total_epistemic = 0.0;

        // Bobot rasa ingin tahu berdasarkan mode kesadaran
        let epistemic_weight = match self.current_mode {
            SimulationMode::StrictVSA => 1.0,
            SimulationMode::Probabilistic => 50.0, // Rasa penasaran meroket!
            SimulationMode::Counterfactual => 100.0,
        };

        // Simulasi masa depan (Horizon Simulation)
        for step in 0..policy.horizon {
            if step < policy.actions.len() {
                let action = &policy.actions[step];
                MultiverseSandbox::apply_axiom(
                    &mut temp_state,
                    &action.condition_tensor,
                    &action.tensor_rule,
                    &action.tensor_rule,
                    action.delta_x,
                    action.delta_y,
                    action.physics_tier,
                    &action.description,
                );
            }

            // 1. Pragmatic: Seberapa cocok dengan Ground Truth?
            // Kita gunakan depth_ratio 0.0 agar hadiah dimensi (-500) langsung aktif
            let pragmatic = SimdEnergyCalculator::calculate_pragmatic_streaming(
                &temp_state,
                expected_grid,
                temp_state.global_width as usize,
                temp_state.global_height as usize,
                &CognitivePhase::MacroStructural,
                1e-6,
            );
            total_pragmatic += pragmatic;

            // 2. Epistemic: Seberapa besar ledakan informasi/perubahan struktur?
            let epistemic = SimdEnergyCalculator::calculate_epistemic(&temp_state, initial_state);
            total_epistemic += epistemic;
        }

        // Expected Free Energy (Semakin negatif/kecil, semakin bagus)
        total_pragmatic - (epistemic_weight * total_epistemic)
    }
}
