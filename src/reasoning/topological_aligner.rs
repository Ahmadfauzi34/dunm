use crate::core::config::GLOBAL_DIMENSION;
use crate::core::core_seeds::CoreSeeds;
use crate::core::entity_manifold::EntityManifold;
use crate::core::fhrr::FHRR;
use ndarray::Array1;
use std::collections::HashMap;

pub struct TopologicalMatch {
    pub source_index: usize,
    pub target_index: i32,
    pub similarity: f32,
    pub condition_tensor: Option<Array1<f32>>, // IF(Condition)
    pub delta_spatial: Array1<f32>,
    pub delta_semantic: Array1<f32>,
    pub delta_x: f32,
    pub delta_y: f32,
    pub axiom_type: String,
    pub physics_tier: u8,
}

pub struct TopologicalAligner;

impl TopologicalAligner {
    pub fn align(
        source_manifold: &EntityManifold,
        target_manifold: &EntityManifold,
    ) -> Vec<TopologicalMatch> {
        let mut matches = Vec::new();

        // Menyimpan voting pergeseran berdasarkan Token Warna tertentu (Kondisional)
        // Format Key: "ColorToken|dx_dy" -> jumlah vote
        let mut conditional_trans_votes: HashMap<String, i32> = HashMap::new();
        // Menyimpan voting perubahan warna secara spesifik
        // Format Key: "ColorToken|targetColor" -> jumlah vote
        let mut conditional_color_votes: HashMap<String, i32> = HashMap::new();

        // RELASIONAL: Menyimpan voting gerakan menuju warna objek tertentu
        // Format Key: "MoverColor|TargetColor" -> jumlah vote
        let mut relational_target_votes: HashMap<String, i32> = HashMap::new();

        for s_idx in 0..source_manifold.active_count {
            let s_color = source_manifold.tokens[s_idx];
            let s_cx = source_manifold.centers_x[s_idx];
            let s_cy = source_manifold.centers_y[s_idx];

            for t_idx in 0..target_manifold.active_count {
                let t_color = target_manifold.tokens[t_idx];
                let t_cx = target_manifold.centers_x[t_idx];
                let t_cy = target_manifold.centers_y[t_idx];

                if s_color == 0 || t_color == 0 {
                    continue;
                }

                let dx = t_cx - s_cx;
                let dy = t_cy - s_cy;

                // Voting Translasi Absolut (Jarak pasti)
                if s_color == t_color {
                    let key = format!("{}|{:.1}_{:.1}", s_color, dx, dy);
                    *conditional_trans_votes.entry(key).or_insert(0) += 1;

                    // Deteksi Relasional: Objek apa yang didekati benda ini di kanvas Output?
                    // Kita scan objek lain di target_manifold yang posisinya sangat dekat dengan piksel ini (t_cx, t_cy)
                    if dx.abs() > 0.0 || dy.abs() > 0.0 {
                        for anchor_idx in 0..target_manifold.active_count {
                            let anchor_color = target_manifold.tokens[anchor_idx];
                            if anchor_color == 0 || anchor_color == s_color {
                                continue;
                            }

                            let ax = target_manifold.centers_x[anchor_idx];
                            let ay = target_manifold.centers_y[anchor_idx];

                            // Jika jarak benda yang bergerak ini berada di radius 1 piksel dari benda lain (Anchor)
                            let dist = (t_cx - ax).abs() + (t_cy - ay).abs();
                            if dist <= 1.0 {
                                let rel_key = format!("{}|{}", s_color, anchor_color);
                                *relational_target_votes.entry(rel_key).or_insert(0) += 1;
                            }
                        }
                    }
                }

                // Voting Warna (Jarak nol, tapi warna berubah)
                if dx.abs() < 0.1 && dy.abs() < 0.1 && s_color != t_color {
                    let key = format!("{}|{}", s_color, t_color);
                    *conditional_color_votes.entry(key).or_insert(0) += 1;
                }
            }
        }

        let mut id_tensor = Array1::<f32>::zeros(GLOBAL_DIMENSION);
        id_tensor[0] = 1.0;
        id_tensor[GLOBAL_DIMENSION - 1] = 1.0;

        // Generate Hypothesis: TRANSLASI ABSOLUT
        let mut sorted_trans: Vec<(&String, &i32)> = conditional_trans_votes.iter().collect();
        sorted_trans.sort_by(|a, b| b.1.cmp(a.1));

        for (idx, (key, _count)) in sorted_trans.iter().enumerate().take(3) {
            let p1: Vec<&str> = key.split('|').collect();
            let color: i32 = p1[0].parse().unwrap_or(0);

            let p2: Vec<&str> = p1[1].split('_').collect();
            let dx: f32 = p2[0].parse().unwrap_or(0.0);
            let dy: f32 = p2[1].parse().unwrap_or(0.0);

            let condition_phase = FHRR::fractional_bind(CoreSeeds::color_seed(), color as f32);

            matches.push(TopologicalMatch {
                source_index: 0,
                target_index: -1,
                similarity: 1.0 - (idx as f32 * 0.1),
                condition_tensor: Some(condition_phase),
                delta_spatial: id_tensor.clone(),
                delta_semantic: id_tensor.clone(),
                delta_x: dx,
                delta_y: dy,
                axiom_type: format!("IF_COLOR({})_THEN_TRANS_{}_{}", color, dx, dy),
                physics_tier: 0,
            });
        }

        // Generate Hypothesis: TRANSLASI RELASIONAL (Menuju Objek Lain)
        let mut sorted_relational: Vec<(&String, &i32)> = relational_target_votes.iter().collect();
        sorted_relational.sort_by(|a, b| b.1.cmp(a.1));

        for (idx, (key, _count)) in sorted_relational.iter().enumerate().take(2) {
            let parts: Vec<&str> = key.split('|').collect();
            let s_color: i32 = parts[0].parse().unwrap_or(0);
            let anchor_color: i32 = parts[1].parse().unwrap_or(0);

            let condition_phase = FHRR::fractional_bind(CoreSeeds::color_seed(), s_color as f32);

            matches.push(TopologicalMatch {
                source_index: 0,
                target_index: -1,
                similarity: 1.0 - (idx as f32 * 0.1),
                condition_tensor: Some(condition_phase),
                delta_spatial: id_tensor.clone(),
                delta_semantic: id_tensor.clone(),
                // Kita simpan target Anchor Color di delta_x sementara (Sebagai flag komputasi Sandbox)
                // Ini menandakan "Hitung dx dy dinamis menuju warna X" (di mana X = delta_x)
                delta_x: anchor_color as f32,
                delta_y: 999.0, // Flag khusus untuk Relasional
                axiom_type: format!("IF_COLOR({})_THEN_MOVE_TO_COLOR({})", s_color, anchor_color),
                physics_tier: 3, // Tier 3 = Relational Physics (Dynamic DX/DY)
            });
        }

        // Generate Hypothesis: MUTASI WARNA
        let mut sorted_color: Vec<(&String, &i32)> = conditional_color_votes.iter().collect();
        sorted_color.sort_by(|a, b| b.1.cmp(a.1));

        for (idx, (key, _count)) in sorted_color.iter().enumerate().take(3) {
            let parts: Vec<&str> = key.split('|').collect();
            let s_color: i32 = parts[0].parse().unwrap_or(0);
            let t_color: i32 = parts[1].parse().unwrap_or(0);

            let condition_phase = FHRR::fractional_bind(CoreSeeds::color_seed(), s_color as f32);
            let tgt_color_phase = FHRR::fractional_bind(CoreSeeds::color_seed(), t_color as f32);
            let d_semantic = FHRR::bind(&tgt_color_phase, &FHRR::inverse(&condition_phase));

            matches.push(TopologicalMatch {
                source_index: 0,
                target_index: -1,
                similarity: 1.0 - (idx as f32 * 0.1),
                condition_tensor: Some(condition_phase),
                delta_spatial: id_tensor.clone(),
                delta_semantic: d_semantic,
                delta_x: 0.0,
                delta_y: 0.0,
                axiom_type: format!("IF_COLOR({})_THEN_SHIFT(->{})", s_color, t_color),
                physics_tier: 0,
            });
        }

        // Generate Hypothesis: GEOMETRI GLOBAL & KONDISIONAL
        // Kita suntikkan aksioma geometri statis untuk dicoba oleh MCTS
        let geometry_ops = [
            "MIRROR_X",
            "MIRROR_Y",
            "ROTATE_90",
            "ROTATE_180",
            "ROTATE_270",
        ];

        // Coba apply secara global (tanpa kondisi warna)
        for (i, op) in geometry_ops.iter().enumerate() {
            matches.push(TopologicalMatch {
                source_index: 0,
                target_index: -1,
                similarity: 0.5 - (i as f32 * 0.01),
                condition_tensor: None, // Berlaku untuk semua objek
                delta_spatial: id_tensor.clone(),
                delta_semantic: id_tensor.clone(),
                delta_x: 0.0,
                delta_y: 0.0,
                axiom_type: format!("GLOBAL_{}", op),
                physics_tier: 4, // Tier 4 = Geometri
            });
        }

        // Coba apply secara kondisional untuk setiap warna dominan di source
        let mut source_colors: Vec<i32> = source_manifold
            .tokens
            .iter()
            .copied()
            .filter(|&c| c > 0)
            .collect();
        source_colors.sort_unstable();
        source_colors.dedup();

        for color in source_colors.iter().take(3) {
            let condition_phase = FHRR::fractional_bind(CoreSeeds::color_seed(), *color as f32);
            for op in geometry_ops.iter() {
                matches.push(TopologicalMatch {
                    source_index: 0,
                    target_index: -1,
                    similarity: 0.4,
                    condition_tensor: Some(condition_phase.clone()),
                    delta_spatial: id_tensor.clone(),
                    delta_semantic: id_tensor.clone(),
                    delta_x: 0.0,
                    delta_y: 0.0,
                    axiom_type: format!("IF_COLOR({})_THEN_{}", color, op),
                    physics_tier: 4, // Tier 4 = Geometri
                });
            }
        }

        // Generate Hypothesis: DESTROY (Annihilation) & CROP
        // Coba hancurkan atau jadikan crop target setiap warna yang ada
        for color in source_colors.iter() {
            let condition_phase = FHRR::fractional_bind(CoreSeeds::color_seed(), *color as f32);

            // Aksioma ERASE (Hancurkan warna ini)
            matches.push(TopologicalMatch {
                source_index: 0,
                target_index: -1,
                similarity: 0.35,
                condition_tensor: Some(condition_phase.clone()),
                delta_spatial: id_tensor.clone(),
                delta_semantic: id_tensor.clone(),
                delta_x: 0.0,
                delta_y: 0.0,
                axiom_type: format!("IF_COLOR({})_THEN_ERASE", color),
                physics_tier: 5, // Tier 5 = Destroy
            });

            // Aksioma CROP (Crop universe ke batas warna ini)
            matches.push(TopologicalMatch {
                source_index: 0,
                target_index: -1,
                similarity: 0.35,
                condition_tensor: Some(condition_phase.clone()),
                delta_spatial: id_tensor.clone(),
                delta_semantic: id_tensor.clone(),
                delta_x: 0.0,
                delta_y: 0.0,
                axiom_type: format!("CROP_TO_COLOR({})", color),
                physics_tier: 7, // Tier 7 = Crop
            });
        }

        // Generate Hypothesis: SPAWN
        // Jika ada warna di target yang TIDAK ADA di source (Warna baru muncul)
        let mut target_colors: Vec<i32> = target_manifold
            .tokens
            .iter()
            .copied()
            .filter(|&c| c > 0)
            .collect();
        target_colors.sort_unstable();
        target_colors.dedup();

        for t_color in target_colors.iter() {
            if !source_colors.contains(t_color) {
                // Warna baru ini pasti di-spawn. Coba tebak bounding box warna apa yang dia tempati.
                for s_color in source_colors.iter() {
                    let condition_phase =
                        FHRR::fractional_bind(CoreSeeds::color_seed(), *s_color as f32);
                    matches.push(TopologicalMatch {
                        source_index: 0,
                        target_index: -1,
                        similarity: 0.6, // Prioritas cukup tinggi jika memang warna baru
                        condition_tensor: Some(condition_phase.clone()),
                        delta_spatial: id_tensor.clone(),
                        delta_semantic: id_tensor.clone(),
                        delta_x: *t_color as f32, // Target color disimpan di delta_x
                        delta_y: 0.0,
                        axiom_type: format!("IF_COLOR({})_THEN_SPAWN_COLOR({})", s_color, t_color),
                        physics_tier: 6, // Tier 6 = Spawn
                    });
                }
            }
        }

        matches.push(TopologicalMatch {
            source_index: 0,
            target_index: -1,
            similarity: 0.1,
            condition_tensor: None,
            delta_spatial: id_tensor.clone(),
            delta_semantic: id_tensor.clone(),
            delta_x: 0.0,
            delta_y: 0.0,
            axiom_type: "IDENTITY_STATIC".to_string(),
            physics_tier: 0,
        });

        matches
    }
}
