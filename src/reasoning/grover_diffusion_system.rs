use crate::core::entity_manifold::EntityManifold;
use crate::reasoning::hierarchical_inference::SimulationMode;
use crate::reasoning::multiverse_sandbox::MultiverseSandbox;
use crate::reasoning::quantum_search_simd::{CognitivePhase, SimdEnergyCalculator};
use ndarray::Array1;

pub struct GroverConfig {
    pub dimensions: usize,
    pub search_space_size: usize,
    pub temperature: f32,
    pub free_energy_threshold: f32,
    pub max_iterations: usize,
}

pub struct GroverCandidate {
    pub energy: f32,
    pub tensor_rule: Array1<f32>,
    pub condition_tensor: Option<Array1<f32>>,
    pub delta_x: f32,
    pub delta_y: f32,
    pub physics_tier: u8,
    pub axiom_type: String,
}

pub struct TrainState {
    pub in_state: EntityManifold,
    pub expected_grid: Vec<Vec<i32>>,
}

/// ============================================================================
/// GROVER DIFFUSION SYSTEM (Real-Valued VSA/FHRR Implementation)
/// ============================================================================
/// Menjalankan algoritma Grover (Amplitude Amplification) menggunakan
/// Termodinamika Berkelanjutan (Continuous Free Energy Oracle).
pub struct GroverDiffusionSystem<'a> {
    pub config: GroverConfig,
    pub amplitudes: Vec<f32>,
    pub multipliers: Vec<f32>,
    pub energies: Vec<f32>,
    mean_buffer: Vec<f32>,
    _sandbox: &'a mut MultiverseSandbox,
}

impl<'a> GroverDiffusionSystem<'a> {
    pub fn new(sandbox: &'a mut MultiverseSandbox, config: GroverConfig) -> Self {
        let total_size = config.search_space_size * config.dimensions;
        Self {
            amplitudes: vec![0.0; total_size],
            multipliers: vec![0.0; config.search_space_size],
            energies: vec![0.0; config.search_space_size],
            mean_buffer: vec![0.0; config.dimensions],
            config,
            _sandbox: sandbox,
        }
    }

    /// MENGINISIALISASI "WARM START" (Hybrid ARC Architecture)
    pub fn warm_start(&mut self, candidates: &[GroverCandidate]) {
        let n = self.config.search_space_size.min(candidates.len());
        let d = self.config.dimensions;

        self.amplitudes.fill(0.0);

        let mut total_initial_energy_sq = 0.0;
        for i in 0..n {
            let energy = candidates[i].energy;
            let base_amp = 0.001_f32.max(energy.sqrt());
            total_initial_energy_sq += base_amp * base_amp;
        }

        let normalization_factor = 1.0 / (total_initial_energy_sq + 1e-15).sqrt();

        for i in 0..n {
            let base_idx = i * d;
            let energy = candidates[i].energy;
            let amp = energy.sqrt() * normalization_factor;

            let rule_tensor = &candidates[i].tensor_rule;

            for dim in 0..d {
                self.amplitudes[base_idx + dim] = rule_tensor[dim] * amp;
            }
        }
    }

    /// CONTINUOUS FREE ENERGY ORACLE
    pub fn evaluate_oracle(
        &mut self,
        candidates: &[GroverCandidate],
        train_states: &[TrainState],
        mode: &SimulationMode,
    ) {
        let n = self.config.search_space_size.min(candidates.len());
        let d = self.config.dimensions;

        let epistemic_weight = match mode {
            SimulationMode::StrictVSA => 1.0,
            SimulationMode::Probabilistic => 50.0,
            SimulationMode::Counterfactual => 100.0,
        };

        // 1. Kalkulasi Energi untuk semua kandidat
        for i in 0..n {
            let candidate = &candidates[i];
            let mut total_free_energy = 0.0;

            for state in train_states {
                let mut temp_state = state.in_state.clone();
                let _dummy_spatial_delta =
                    Array1::<f32>::zeros(crate::core::config::GLOBAL_DIMENSION);

                MultiverseSandbox::apply_axiom(
                    &mut temp_state,
                    &candidate.condition_tensor,
                    &candidate.tensor_rule,
                    &candidate.tensor_rule,
                    candidate.delta_x,
                    candidate.delta_y,
                    candidate.physics_tier,
                    &candidate.axiom_type,
                );

                let pragmatic_error = SimdEnergyCalculator::calculate_pragmatic_streaming(
                    &temp_state,
                    &state.expected_grid,
                    temp_state.global_width as usize,
                    temp_state.global_height as usize,
                    &CognitivePhase::MacroStructural, // Grover selalu beroperasi di fase penentCognitivePhase::MacroStructural,
                    1e-6,
                );

                // Epistemic Value (Penghancuran sampah kosmik = bonus tinggi!)
                let epistemic_value =
                    SimdEnergyCalculator::calculate_epistemic(&temp_state, &state.in_state);

                // G(π)
                total_free_energy += pragmatic_error - (epistemic_weight * epistemic_value);
            }
            self.energies[i] = total_free_energy;
        }

        // 2. Cari Batas Min dan Max Energi di Populasi (Adaptive Bounds)
        let mut min_e = f32::MAX;
        let mut max_e = f32::MIN;
        for i in 0..n {
            let e = self.energies[i];
            if e < min_e {
                min_e = e;
            }
            if e > max_e {
                max_e = e;
            }
        }

        // 3. Terapkan Multiplier via Linear Mapping (Anti-Vanishing Gradient)
        let range = (max_e - min_e).max(1e-5); // Hindari pembagian dengan nol
        for i in 0..n {
            let e = self.energies[i];

            // Normalisasi: 0.0 (Energi Terendah/Terbaik) ke 1.0 (Energi Tertinggi/Terburuk)
            let normalized = (e - min_e) / range;

            // Grover Phase Inversion Mapping:
            // 0.0 (Terbaik) -> -1.0 (Inversi Amplitudo Penuh)
            // 1.0 (Terburuk) ->  1.0 (Tidak Ada Inversi)
            // Kita gunakan power/akar (opsional) untuk "menajamkan" kandidat terbaik jika perlu
            self.multipliers[i] = -1.0 + (2.0 * normalized);
        }

        // Apply Multiplier ke Amplitudo Kuantum
        for i in 0..n {
            let mult = self.multipliers[i];
            let base_idx = i * d;

            for dim in 0..d {
                self.amplitudes[base_idx + dim] *= mult;
            }
        }
    }

    /// DIFFUSION OPERATOR (Inversion About Mean)
    pub fn apply_diffusion(&mut self, n: usize) {
        let d = self.config.dimensions;

        self.mean_buffer.fill(0.0);
        for i in 0..n {
            let base_idx = i * d;
            for dim in 0..d {
                self.mean_buffer[dim] += self.amplitudes[base_idx + dim];
            }
        }

        let inv_n = 1.0 / (n as f32);
        for dim in 0..d {
            self.mean_buffer[dim] *= inv_n;
        }

        // Reflection
        for i in 0..n {
            let base_idx = i * d;
            for dim in 0..d {
                let mean = self.mean_buffer[dim];
                self.amplitudes[base_idx + dim] = 2.0 * mean - self.amplitudes[base_idx + dim];
            }
        }

        self.thermal_normalize(n);
    }

    /// Normalisasi Energi Kinetik menggunakan distribusi Boltzmann tiruan.
    fn thermal_normalize(&mut self, n: usize) {
        let d = self.config.dimensions;
        let t = self.config.temperature;

        let mut norms = vec![0.0; n];

        for i in 0..n {
            let base_idx = i * d;
            let mut sum_sq = 0.0;
            for dim in 0..d {
                let a = self.amplitudes[base_idx + dim];
                sum_sq += a * a;
            }
            norms[i] = sum_sq.sqrt();
        }

        for i in 0..n {
            let base_idx = i * d;
            let norm = norms[i] + 1e-10;

            let thermal_factor = (-norm / t).exp();
            let scale = 1.0 / (norm + thermal_factor);

            for dim in 0..d {
                self.amplitudes[base_idx + dim] *= scale;
            }
        }
    }

    /// Eksekusi Amplifikasi Kuantum (Grover Iteration)
    pub fn search(
        &mut self,
        candidates: &[GroverCandidate],
        train_states: &[TrainState],
        mode: &SimulationMode,
    ) -> Option<usize> {
        let n = self.config.search_space_size.min(candidates.len());
        if n == 0 {
            return None;
        }

        self.warm_start(candidates);

        // K_opt
        let mut iterations = ((std::f32::consts::PI / 4.0) * (n as f32).sqrt()).ceil() as usize;
        iterations = iterations.min(self.config.max_iterations);

        for _ in 0..iterations {
            self.evaluate_oracle(candidates, train_states, mode);
            self.apply_diffusion(n);
        }

        let mut max_amp = -9999.0;
        let mut winner_idx = 0;

        for i in 0..n {
            let base_idx = i * self.config.dimensions;
            let mut state_energy = 0.0;

            for dim in 0..self.config.dimensions {
                let a = self.amplitudes[base_idx + dim];
                state_energy += a * a;
            }

            if state_energy > max_amp {
                max_amp = state_energy;
                winner_idx = i;
            }
        }

        Some(winner_idx)
    }
}
