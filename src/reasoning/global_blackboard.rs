use crate::core::config::GLOBAL_DIMENSION;
use crate::core::fhrr::FHRR;
use ndarray::Array1;

/// ============================================================================
/// GLOBAL BLACKBOARD (Collective Consciousness)
/// 100% Branchless | Superposisi Multi-Agen
/// ============================================================================
/// Tempat di mana berbagai agen (Visual, Logika, Spasial) menyatukan
/// pemikiran mereka menjadi satu gelombang kesadaran kolektif.
pub struct GlobalBlackboard {
    collective_state: Array1<f32>,
}

impl Default for GlobalBlackboard {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalBlackboard {
    pub fn new() -> Self {
        Self {
            collective_state: Array1::zeros(GLOBAL_DIMENSION),
        }
    }

    /// 🌐 SYNCHRONIZE (Zero If-Else)
    /// Menyatukan pemikiran dari berbagai Agen menjadi satu Kesadaran Kolektif.
    pub fn synchronize(&mut self, agent_states: &[&Array1<f32>]) {
        // 1. Reset state (mengosongkan pikiran kolektif)
        self.collective_state.fill(0.0);

        // 2. Superposisi semua pemikiran agen (Interferensi Konstruktif)
        for state in agent_states {
            for i in 0..GLOBAL_DIMENSION {
                self.collective_state[i] += state[i];
            }
        }

        // 3. Stabilisasi ke Unit Circle (Renormalisasi L2)
        let mut mag_sq = 0.0;
        for i in 0..GLOBAL_DIMENSION {
            mag_sq += self.collective_state[i] * self.collective_state[i];
        }

        // Math Branchless Normalization
        let inv_mag = 1.0 / (mag_sq.sqrt() + 1e-15);
        for i in 0..GLOBAL_DIMENSION {
            self.collective_state[i] *= inv_mag;
        }
    }

    /// Membaca kesadaran kolektif saat ini.
    pub fn read_collective_state(&self) -> &Array1<f32> {
        &self.collective_state
    }

    /// 🌌 CONTEXTUALIZE
    /// Mengikat (Bind) pemikiran agen individu dengan kesadaran kolektif.
    /// Ini membuat agen individu "sadar" akan apa yang dipikirkan agen lain.
    pub fn contextualize_agent(&self, agent_state: &Array1<f32>) -> Array1<f32> {
        // Menggunakan binding sirkular FHRR untuk menggabungkan state individu dengan state kolektif
        let mut bound = FHRR::bind(agent_state, &self.collective_state);

        // Renormalisasi L2 (Math Branchless)
        let mut mag_sq: f32 = 0.0;
        for i in 0..GLOBAL_DIMENSION {
            mag_sq += bound[i] * bound[i];
        }

        let inv_mag = 1.0 / (mag_sq.sqrt() + 1e-15);
        for i in 0..GLOBAL_DIMENSION {
            bound[i] *= inv_mag;
        }

        bound
    }
}
