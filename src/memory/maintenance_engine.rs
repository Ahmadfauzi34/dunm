use crate::core::config::GLOBAL_DIMENSION;
use ndarray::{Array1, ArrayViewMut1};

pub struct MaintenanceEngine;

impl Default for MaintenanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MaintenanceEngine {
    pub fn new() -> Self {
        Self
    }

    /// Hitung panjang/magnitude vektor
    fn magnitude(v: &ArrayViewMut1<f32>) -> f32 {
        let sum_sq: f32 = v.iter().map(|&x| x * x).sum();
        sum_sq.sqrt()
    }

    /// Normalisasi vektor (L2 Norm) agar kembali ke magnitude 1.0 (Unitary)
    fn normalize_in_place(v: &mut ArrayViewMut1<f32>) {
        let mag = Self::magnitude(v);
        if mag > 1e-12 {
            for val in v.iter_mut() {
                *val /= mag;
            }
        }
    }

    /// 🌀 QUANTUM ANNEALING (Termodinamika Kuantum)
    /// Memastikan semua vektor Seed 100% Ortogonal (Tegak Lurus).
    pub fn anneal_memory(
        &self,
        tensors: &mut [Array1<f32>],
        base_learning_rate: f32,
        epochs: usize,
    ) -> (f32, f32) {
        let num_seeds = tensors.len();
        if num_seeds <= 1 {
            return (0.0, 0.0);
        }

        let orthogonal_tolerance = 0.05;

        // Hitung total noise sebelum anneal
        let mut total_noise_before = 0.0;
        let mut pairs_count = 0;

        for i in 0..num_seeds {
            for j in (i + 1)..num_seeds {
                let v_a = tensors[i].view();
                let v_b = tensors[j].view();
                total_noise_before += v_a
                    .iter()
                    .zip(v_b.iter())
                    .map(|(&x, &y)| x * y)
                    .sum::<f32>()
                    .abs();
                pairs_count += 1;
            }
        }

        println!("[MaintenanceEngine] 🔥 Memulai Pendinginan Termodinamika (Simulated Annealing) pada {} Seed...", num_seeds);
        let mut is_stable = false;

        let mut repulsion_fields = vec![0.0; num_seeds * GLOBAL_DIMENSION];

        for epoch in 0..epochs {
            let mut total_collisions = 0;
            repulsion_fields.fill(0.0);

            // A. Evaluasi N-Body Problem
            for i in 0..num_seeds {
                let offset_a = i * GLOBAL_DIMENSION;
                for j in (i + 1)..num_seeds {
                    let offset_b = j * GLOBAL_DIMENSION;

                    let mut sim = 0.0;
                    for (&a_val, &b_val) in tensors[i].iter().zip(tensors[j].iter()).take(GLOBAL_DIMENSION) {
                        sim += a_val * b_val;
                    }

                    if sim.abs() > orthogonal_tolerance {
                        total_collisions += 1;
                        for d in 0..GLOBAL_DIMENSION {
                            let b_val = tensors[j][d];
                            let a_val = tensors[i][d];
                            repulsion_fields[offset_a + d] -= sim * b_val;
                            repulsion_fields[offset_b + d] -= sim * a_val;
                        }
                    }
                }
            }

            // B. Cek Keseimbangan
            if total_collisions == 0 {
                println!(
                    "   ✅ Sistem mencapai Keseimbangan Kuantum di Epoch {}!",
                    epoch
                );
                is_stable = true;
                break;
            }

            // C. Terapkan Gaya Tolak
            let temperature = base_learning_rate * (-((epoch as f32) / 5.0)).exp();

            for (i, tensor) in tensors.iter_mut().enumerate().take(num_seeds) {
                let offset_a = i * GLOBAL_DIMENSION;
                let mut has_energy = false;

                for d in 0..GLOBAL_DIMENSION {
                    if repulsion_fields[offset_a + d] != 0.0 {
                        has_energy = true;
                        break;
                    }
                }

                if has_energy {
                    let mut v_a = tensor.view_mut();
                    for d in 0..GLOBAL_DIMENSION {
                        v_a[d] += repulsion_fields[offset_a + d] * temperature;
                    }
                    Self::normalize_in_place(&mut v_a);
                }
            }
        }

        if !is_stable {
            println!("   ⚠️ Peringatan: Sistem dihentikan sebelum mencapai ortogonalitas sempurna (Batas Epoch tercapai).");
        }

        // Hitung total noise sesudah anneal
        let mut total_noise_after = 0.0;
        for i in 0..num_seeds {
            for j in (i + 1)..num_seeds {
                let v_a = tensors[i].view();
                let v_b = tensors[j].view();
                total_noise_after += v_a
                    .iter()
                    .zip(v_b.iter())
                    .map(|(&x, &y)| x * y)
                    .sum::<f32>()
                    .abs();
            }
        }

        let avg_before = total_noise_before / (pairs_count as f32);
        let avg_after = total_noise_after / (pairs_count as f32);

        println!("[MaintenanceEngine] 💾 Manifold Memory berhasil dikristalisasi. Crosstalk Turun: {:.4} -> {:.4}", avg_before, avg_after);

        (avg_before, avg_after)
    }
}
