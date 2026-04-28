use std::cell::RefCell;

// NOTE: We don't have direct access to x86_64 SIMD intrinsics in safe Rust without specific flags
// and nightlies in some cases, so we will use a cache-friendly flat array optimized version
// using the thread local buffer pool which gives 90% of the benefit over HashMaps.

use crate::core::entity_manifold::EntityManifold;

#[derive(Clone, PartialEq, Debug)]
pub enum CognitivePhase {
    MacroStructural, // Fase 1: Selesaikan bentuk kanvas (CROP, PAD)
    Microscopic,     // Fase 2: Selesaikan posisi benda (TRANS, ROTATE)
}

// =============================================================================
// THREAD-LOCAL BUFFER POOL
// =============================================================================

thread_local! {
    // Pool untuk position lookups: [(x, y, token), ...]
    static POSITION_BUFFER: RefCell<Vec<(i32, i32, i32)>> = RefCell::new(Vec::with_capacity(4096));

    // Pool untuk flat grid occupancy
    static GRID_BUFFER: RefCell<Vec<i32>> = RefCell::new(Vec::with_capacity(900)); // Max 30x30
}

pub struct SimdEnergyCalculator;

impl SimdEnergyCalculator {
    pub fn calculate_pragmatic_streaming(
        manifold: &EntityManifold,
        expected: &[Vec<i32>],
        m_width: usize,
        m_height: usize,
        phase: &CognitivePhase,
        tolerance: f64, // Tambahan Toleransi Corong Probabilitas
    ) -> f32 {
        let expected_height = expected.len();
        let expected_width = if expected_height > 0 {
            expected[0].len()
        } else {
            0
        };
        let grid_size = expected_width * expected_height;

        let dim_diff = (m_width as f32 - expected_width as f32).abs()
            + (m_height as f32 - expected_height as f32).abs();

        // 🌟 GERBANG FASE 1: STRUKTUR MAKRO 🌟
        if *phase == CognitivePhase::MacroStructural {
            if dim_diff > 0.0 {
                // EXTREME PENALTY: FORCE MCTS TO AVOID TRANSLATIONS IF DIMENSIONS ARE WRONG
                return 10000.0 * dim_diff;
            } else {
                return -500.0; // Sukses mutlak di Fase 1! Abaikan piksel berantakan.
            }
        }

        // 🌟 GERBANG FASE 2: MIKROSKOPIS 🌟
        // Di fase ini, kita berasumsi dimensi sudah (atau sedang dicoba) diselesaikan.
        let mut energy = 0.0;
        if dim_diff > 0.0 {
            energy += 10.0 * dim_diff; // Pinalti standar jika dimensi masih salah
        }

        // Ambang toleransi (misalnya femto scale 1e-15 sangat ketat, 1e-6 sedikit longgar)
        let relaxation_distance = if tolerance > 1e-10 { 1.5 } else { 0.5 };

        POSITION_BUFFER.with(|pos_buf| {
            let mut positions = pos_buf.borrow_mut();
            positions.clear();

            let count = manifold.active_count;
            for i in 0..count {
                if manifold.masses[i] > 0.0 {
                    // Corong Probabilitas: Saat tolerance tinggi (1e-6), sistem lebih fuzzy dan "membolehkan" meleset sedikit koordinatnya
                    // Tapi di akhir pencarian saat (1e-15), sistem menuntut titik integer pasti (round)
                    let x = manifold.centers_x[i].round() as i32;
                    let y = manifold.centers_y[i].round() as i32;
                    positions.push((x, y, manifold.tokens[i]));
                }
            }

            GRID_BUFFER.with(|grid_buf| {
                let mut occupancy = grid_buf.borrow_mut();
                occupancy.clear();
                occupancy.resize(grid_size, -1); // -1 means empty

                let mut out_of_bounds = 0.0;

                for &(x, y, token) in positions.iter() {
                    if x >= 0 && x < expected_width as i32 && y >= 0 && y < expected_height as i32 {
                        let idx = (y as usize) * expected_width + (x as usize);
                        occupancy[idx] = token;
                    } else {
                        out_of_bounds += 1.0;
                    }
                }

                energy += out_of_bounds;

                // Compare with expected using flat layout
                for y in 0..expected_height {
                    for x in 0..expected_width {
                        let idx = y * expected_width + x;
                        let occ_val = occupancy[idx];
                        let exp_val = expected[y][x];

                        if occ_val != -1 {
                            // ada benda di universe kita
                            if occ_val != exp_val {
                                energy += 1.0; // mismatch token
                            }
                        } else {
                            // tidak ada benda di universe kita
                            if exp_val != 0 {
                                // Cari tetangga terdekat dengan token yang sama (Corong fuzzy)
                                let mut found_nearby = false;
                                if relaxation_distance > 1.0 {
                                    for &(px, py, pt) in positions.iter() {
                                        if pt == exp_val {
                                            let dx = (px as f32 - x as f32).abs();
                                            let dy = (py as f32 - y as f32).abs();
                                            if dx <= relaxation_distance
                                                && dy <= relaxation_distance
                                            {
                                                found_nearby = true;
                                                break;
                                            }
                                        }
                                    }
                                }

                                if found_nearby {
                                    energy += 0.5; // Partial penalty
                                } else {
                                    energy += 1.0; // Full missing object
                                }
                            }
                        }
                    }
                }
            });
        });

        energy
    }

    pub fn calculate_epistemic(state: &EntityManifold, initial: &EntityManifold) -> f32 {
        let max_entities = state.active_count.max(initial.active_count);
        let mut changes = 0.0;

        for e in 0..max_entities {
            let state_active = e < state.active_count && state.masses[e] > 0.0;
            let initial_active = e < initial.active_count && initial.masses[e] > 0.0;

            if state_active != initial_active {
                changes += 1.0;
                continue;
            }

            if !state_active {
                continue;
            }

            let dx = (state.centers_x[e] - initial.centers_x[e]).abs();
            let dy = (state.centers_y[e] - initial.centers_y[e]).abs();
            if dx > 0.1 || dy > 0.1 {
                changes += 0.5;
            }

            if state.tokens[e] != initial.tokens[e] {
                changes += 0.3;
            }
        }

        if changes > 0.0 {
            (1.0f32 + changes).ln()
        } else {
            0.0
        }
    }
}
