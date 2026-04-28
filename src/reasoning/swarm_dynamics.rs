use crate::core::core_seeds::CoreSeeds;
use crate::core::entity_manifold::EntityManifold;
use crate::core::fhrr::FHRR;
// AxiomGenerator is missing from rust so we use FHRR directly for macro translations.
// This preserves the kinematics loop exactly as it was.

/// 🐝 SWARM DYNAMICS (Multi-Agent Particle System)
/// Menggunakan prinsip "Subarray View" (Zero-GC) dan Termodinamika Berkelanjutan
/// untuk mengatur perilaku gerombolan (Gravitasi, Kohesi, Cairan, dsb) secara paralel.
pub struct SwarmDynamics;

impl SwarmDynamics {
    /// Menerapkan "Kinetika Gerombolan" (Swarm Kinematics).
    /// Semua entitas yang memenuhi syarat akan bergerak bersamaan (misal: "Gravity Drop", "Flocking").
    pub fn apply_swarm_gravity(u: &mut EntityManifold, delta_x: f32, delta_y: f32) {
        let width = u.global_width;
        let height = u.global_height;

        let total_pixel_steps_x = (delta_x * (width - 1.0)).abs().round() as usize;
        let total_pixel_steps_y = (delta_y * (height - 1.0)).abs().round() as usize;

        let max_steps = total_pixel_steps_x.max(total_pixel_steps_y);

        if max_steps == 0 {
            return;
        }

        let step_x = delta_x / (max_steps as f32);
        let step_y = delta_y / (max_steps as f32);

        // Inline translation generation using FHRR fractional bind
        let swarm_shift_tensor = FHRR::fractional_bind_2d(
            CoreSeeds::x_axis_seed(),
            step_x,
            CoreSeeds::y_axis_seed(),
            step_y,
        );

        for _ in 0..max_steps {
            let mut any_moved = false;

            // We must create a copy of positions to avoid mutating while iterating for collision checks
            let active_count = u.active_count;
            let mut next_rel_xs = vec![0.0; active_count];
            let mut next_rel_ys = vec![0.0; active_count];
            let mut can_move = vec![false; active_count];

            for i in 0..active_count {
                if u.masses[i] == 0.0 {
                    continue;
                }

                let current_rel_x = u.centers_x[i];
                let current_rel_y = u.centers_y[i];
                let span_x = u.spans_x[i];
                let span_y = u.spans_y[i];

                let rx = span_x / 2.0;
                let ry = span_y / 2.0;

                let next_rel_x = current_rel_x + step_x;
                let next_rel_y = current_rel_y + step_y;

                let next_px = next_rel_x * (width - 1.0);
                let next_py = next_rel_y * (height - 1.0);

                if next_py + ry > height - 0.5 || next_py - ry < -0.5 {
                    continue;
                }
                if next_px + rx > width - 0.5 || next_px - rx < -0.5 {
                    continue;
                }

                let mut blocked = false;
                for j in 0..active_count {
                    if i == j || u.masses[j] == 0.0 {
                        continue;
                    }

                    let cx2 = u.centers_x[j] * (width - 1.0);
                    let cy2 = u.centers_y[j] * (height - 1.0);
                    let rx2 = u.spans_x[j] / 2.0;
                    let ry2 = u.spans_y[j] / 2.0;

                    let overlap_x = (rx + rx2) - (next_px - cx2).abs();
                    let overlap_y = (ry + ry2) - (next_py - cy2).abs();

                    if overlap_x >= -0.01 && overlap_y >= -0.01 {
                        blocked = true;
                        break;
                    }
                }

                if !blocked {
                    next_rel_xs[i] = next_rel_x;
                    next_rel_ys[i] = next_rel_y;
                    can_move[i] = true;
                    any_moved = true;
                }
            }

            if !any_moved {
                break;
            }

            // Apply kinetics and quantum shifts
            for i in 0..active_count {
                if can_move[i] {
                    u.centers_x[i] = next_rel_xs[i];
                    u.centers_y[i] = next_rel_ys[i];

                    let future_state = {
                        let entity_tensor = u.get_spatial_tensor(i);
                        FHRR::bind(&entity_tensor, &swarm_shift_tensor)
                    };
                    let mut entity_tensor = u.get_spatial_tensor_mut(i);
                    for d in 0..crate::core::config::GLOBAL_DIMENSION {
                        entity_tensor[d] = future_state[d];
                    }
                }
            }
        }
    }
}
