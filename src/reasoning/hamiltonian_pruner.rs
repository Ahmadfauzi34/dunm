use crate::core::entity_manifold::EntityManifold;
use crate::core::fhrr::FHRR;
use ndarray::Array1;

pub struct Hypothesis {
    pub condition_tensor: Option<Array1<f32>>, // SIAPA yang dikenai rule ini?
    pub tensor_spatial: Array1<f32>,
    pub tensor_semantic: Array1<f32>,
    pub free_energy: f32,
    pub description: String,
    pub delta_x: f32,
    pub delta_y: f32,
    pub depth: u32,
    pub physics_tier: u8,
}

pub struct HamiltonianPruner {
    pub hypotheses: Vec<Hypothesis>,
    pub max_branches: usize,
}

impl Default for HamiltonianPruner {
    fn default() -> Self {
        Self::new()
    }
}

impl HamiltonianPruner {
    pub fn new() -> Self {
        Self {
            hypotheses: Vec::new(),
            max_branches: 1024,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn inject_hypothesis(
        &mut self,
        desc: &str,
        condition: Option<Array1<f32>>,
        t_spatial: &Array1<f32>,
        t_semantic: &Array1<f32>,
        dx: f32,
        dy: f32,
        energy: f32,
        depth: u32,
        physics_tier: u8,
    ) {
        // Cek duplikasi
        for hyp in &self.hypotheses {
            let sim_sp = FHRR::similarity(&hyp.tensor_spatial, t_spatial);
            let sim_sem = FHRR::similarity(&hyp.tensor_semantic, t_semantic);

            let mut cond_match = false;
            if let (Some(c1), Some(c2)) = (&hyp.condition_tensor, &condition) {
                if FHRR::similarity(c1, c2) > 0.99 {
                    cond_match = true;
                }
            } else if hyp.condition_tensor.is_none() && condition.is_none() {
                cond_match = true;
            }

            const EPSILON: f32 = 1e-6;
            if sim_sp > 0.99
                && sim_sem > 0.99
                && cond_match
                && (hyp.delta_x - dx).abs() < EPSILON
                && (hyp.delta_y - dy).abs() < EPSILON
                && hyp.physics_tier == physics_tier
            {
                return; // Sudah ada
            }
        }

        self.hypotheses.push(Hypothesis {
            condition_tensor: condition,
            tensor_spatial: t_spatial.clone(),
            tensor_semantic: t_semantic.clone(),
            free_energy: energy,
            description: desc.to_string(),
            delta_x: dx,
            delta_y: dy,
            depth,
            physics_tier,
        });

        self.enforce_dissipation();
    }

    pub fn calculate_free_energy(actual: &[Vec<i32>], expected: &[Vec<i32>]) -> f32 {
        let mut energy = 0.0;

        if actual.len() != expected.len()
            || (!actual.is_empty() && actual[0].len() != expected[0].len())
        {
            energy += 9999.0;
            return energy;
        }

        let height = actual.len();
        let width = if height > 0 { actual[0].len() } else { 0 };

        for (y, actual_row) in actual.iter().enumerate().take(height) {
            for (x, &actual_val) in actual_row.iter().enumerate().take(width) {
                if actual_val != expected[y][x] {
                    energy += 1.0;
                }
            }
        }

        energy
    }

    /// Menghitung Energy/Error murni tanpa pernah membuat grid intermediat (Zero-Allocation).
    /// Menggunakan sistem sparse `covered` array di stack atau buffer re-use lokal.
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_energy_streaming(
        manifold: &EntityManifold,
        expected: &[Vec<i32>],
        m_width: usize,
        m_height: usize,
        depth_ratio: f32,
    ) -> f32 {
        let mut energy = 0.0;
        let expected_height = expected.len();
        let expected_width = if expected_height > 0 {
            expected[0].len()
        } else {
            0
        };

        // Hukum Pengecualian Pruner: Jika dimensi tidak cocok secara makroskopis,
        // Pinalti Dimensi Adaptif: Kecil di awal iterasi, mematikan di akhir (depth ratio mendekati 1.0)
        if m_width != expected_width || m_height != expected_height {
            let dim_diff = (m_width as f32 - expected_width as f32).abs()
                + (m_height as f32 - expected_height as f32).abs();
            let penalty_multiplier = 10.0 * depth_ratio.max(0.1); // Minimal 0.1 agar tetap terarah
            energy += penalty_multiplier * dim_diff;
        } else {
            // 🌟 HADIAH DIMENSI (DIMENSION REWARD) 🌟
            // Jika dimensi MATCH (contoh: 6x6 == 6x6 setelah di-CROP),
            // berikan diskon energi yang masif di fase awal.
            energy -= 500.0 * (1.0 - depth_ratio);

            // 🌟 PRECISION MODULATION (ACTIVE INFERENCE) 🌟
            if depth_ratio < 0.5 {
                return energy;
            }
        }

        // Flat array untuk menandai piksel mana saja di universe yang sudah tertutupi
        let mut covered = vec![false; expected_width * expected_height];

        // 1. Evaluasi Partikel di Universe terhadap Ground Truth
        for e in 0..manifold.active_count {
            if manifold.masses[e] == 0.0 {
                continue;
            }

            let cx_f = manifold.centers_x[e].round();
            let cy_f = manifold.centers_y[e].round();
            let token = manifold.tokens[e];

            let in_bounds = cx_f >= 0.0
                && cy_f >= 0.0
                && (cx_f as usize) < expected_width
                && (cy_f as usize) < expected_height;

            if in_bounds {
                let cx = cx_f as usize;
                let cy = cy_f as usize;
                let idx = cy * expected_width + cx;
                let expected_token = expected[cy][cx];

                if expected_token != token {
                    energy += 1.0; // Ada benda tapi warnanya beda (Mismatch)
                }
                covered[idx] = true;
            } else {
                energy += 1.0; // Ada partikel keluar angkasa (Luar dimensi)
            }
        }

        // 2. Evaluasi Sisa Ruang Vakum (Apakah ada ruang yang seharusnya berisi objek tapi kita tidak merendernya?)
        for (y, expected_row) in expected.iter().enumerate().take(expected_height) {
            for (x, &expected_val) in expected_row.iter().enumerate().take(expected_width) {
                let idx = y * expected_width + x;
                // Jika posisi ini tidak ter-cover oleh partikel universe KITA,
                // tapi expected-nya BUKAN 0 (artinya seharusnya ada benda di sini!)
                if !covered[idx] && expected_val != 0 {
                    energy += 1.0; // Pinalti karena hilangnya sebuah objek / partikel
                }
            }
        }

        energy
    }

    /// Menyortir hipotesis berdasarkan free energy dan memotong yang melebihi max_branches.
    ///
    /// # Panics
    /// Fungsi ini tidak seharusnya panic pada kondisi normal.
    pub fn enforce_dissipation(&mut self) {
        self.hypotheses
            .sort_by(|a, b| a.free_energy.total_cmp(&b.free_energy));
        if self.hypotheses.len() > self.max_branches {
            self.hypotheses.truncate(self.max_branches);
        }
    }

    pub fn clear_all(&mut self) {
        self.hypotheses.clear();
    }

    pub fn extract_ground_state(&self) -> Option<&Hypothesis> {
        self.hypotheses.first()
    }
        }
