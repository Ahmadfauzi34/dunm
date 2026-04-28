use crate::core::config::GLOBAL_DIMENSION;
use crate::core::core_seeds::CoreSeeds;
use crate::core::entity_manifold::EntityManifold;
use crate::core::fhrr::FHRR;
use crate::perception::hierarchical_gestalt::{AtomType, GestaltAtom, GestaltEngine};
use crate::perception::relational_hierarchy::{
    RelationalEngine, RelationalStructure, StructureType,
};
use crate::reasoning::topological_aligner::TopologicalMatch;
use ndarray::Array1;

/// Generator aksioma berbasis hierarki Top-Down (tidak brute-force!)
pub struct TopDownAxiomator;

impl TopDownAxiomator {
    /// Generate aksioma yang masuk akal berdasarkan struktur yang terdeteksi
    pub fn generate_axioms(
        input_manifold: &EntityManifold,
        output_manifold: &EntityManifold,
    ) -> Vec<TopologicalMatch> {
        let mut axioms = Vec::new();

        // Level 1: Extract Gestalt Atoms
        let input_atoms = GestaltEngine::extract_atoms(input_manifold);
        let output_atoms = GestaltEngine::extract_atoms(output_manifold);

        // Level 2: Analyze Relations
        let input_structures = RelationalEngine::analyze_relations(&input_atoms);
        let output_structures = RelationalEngine::analyze_relations(&output_atoms);

        // Generate targeted axioms berdasarkan perubahan struktural
        axioms.extend(Self::generate_fill_axioms(&input_atoms, &output_atoms));
        axioms.extend(Self::generate_container_axioms(
            &input_structures,
            &output_structures,
        ));
        axioms.extend(Self::generate_symmetry_axioms(&input_atoms, &output_atoms));
        axioms.extend(Self::generate_grid_axioms(
            &input_structures,
            &output_structures,
        ));
        axioms.extend(Self::generate_movement_axioms(&input_atoms, &output_atoms));

        // CROP PHYSICS INJECTION
        axioms.extend(Self::generate_macroscopic_crop_axioms(
            input_manifold,
            output_manifold,
        ));

        // Prioritize: sort by structural confidence
        axioms.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        axioms
    }

    /// Deteksi: "Dimensi kanvas mengecil secara makroskopis" -> CROP_TO_COLOR
    fn generate_macroscopic_crop_axioms(
        input_manifold: &EntityManifold,
        output_manifold: &EntityManifold,
    ) -> Vec<TopologicalMatch> {
        let mut axioms = Vec::new();

        let in_w = input_manifold.global_width;
        let in_h = input_manifold.global_height;
        let out_w = output_manifold.global_width;
        let out_h = output_manifold.global_height;

        // Jika dimensi mengecil, ini pasti ada mekanisme CROP
        if out_w < in_w || out_h < in_h {
            let mut colors_in_input = Vec::new();
            for i in 0..input_manifold.active_count {
                if input_manifold.masses[i] > 0.0 && input_manifold.tokens[i] > 0 {
                    colors_in_input.push(input_manifold.tokens[i]);
                }
            }
            colors_in_input.sort_unstable();
            colors_in_input.dedup();

            // Uji setiap warna: Apakah rentang (span) warna ini cocok dengan dimensi output?
            for color in colors_in_input {
                let mut min_x = 9999.0;
                let mut max_x = -9999.0;
                let mut min_y = 9999.0;
                let mut max_y = -9999.0;
                let mut found = false;

                for i in 0..input_manifold.active_count {
                    if input_manifold.masses[i] > 0.0 && input_manifold.tokens[i] == color {
                        found = true;
                        let cx = input_manifold.centers_x[i] * (in_w - 1.0); // cx in pixels
                        let cy = input_manifold.centers_y[i] * (in_h - 1.0); // cy in pixels
                        if cx < min_x {
                            min_x = cx;
                        }
                        if cx > max_x {
                            max_x = cx;
                        }
                        if cy < min_y {
                            min_y = cy;
                        }
                        if cy > max_y {
                            max_y = cy;
                        }
                    }
                }

                if found {
                    let span_w = (max_x - min_x) + 1.0;
                    let span_h = (max_y - min_y) + 1.0;

                    // Jika rentang warna ini cocok dengan ukuran output target, ini adalah jangkar CROP!
                    if (span_w - out_w).abs() <= 1.0 && (span_h - out_h).abs() <= 1.0 {
                        let mut condition_phase =
                            Array1::<f32>::zeros(crate::core::config::GLOBAL_DIMENSION);
                        let color_phase =
                            FHRR::fractional_bind(CoreSeeds::color_seed(), color as f32);
                        for i in 0..crate::core::config::GLOBAL_DIMENSION {
                            condition_phase[i] = color_phase[i];
                        }

                        axioms.push(TopologicalMatch {
                            source_index: 0,
                            target_index: -1,
                            similarity: 0.98, // Prioritas Absolut/Sangat Tinggi!
                            condition_tensor: Some(condition_phase.clone()),
                            delta_spatial: Self::identity_tensor(),
                            delta_semantic: Self::identity_tensor(),
                            delta_x: 0.0,
                            delta_y: 0.0,
                            axiom_type: format!("CROP_TO_COLOR({})", color),
                            physics_tier: 7, // Tier 7 = Crop Physics
                        });
                    }
                }

                // 🌟 AKSIOMA BARU: CROP_WINDOW_AROUND (Anchor-Based Crop)
                // Usulkan warna ini sebagai titik pusat untuk jendela out_w x out_h
                let mut condition_phase =
                    Array1::<f32>::zeros(crate::core::config::GLOBAL_DIMENSION);
                let color_phase = FHRR::fractional_bind(CoreSeeds::color_seed(), color as f32);
                for i in 0..crate::core::config::GLOBAL_DIMENSION {
                    condition_phase[i] = color_phase[i];
                }

                axioms.push(TopologicalMatch {
                    source_index: 0,
                    target_index: -1,
                    similarity: 0.98, // VIP Pass prioritas tinggi untuk Depth 1
                    condition_tensor: Some(condition_phase),
                    delta_spatial: Self::identity_tensor(),
                    delta_semantic: Self::identity_tensor(),
                    delta_x: 0.0, // Dinamika Kuantum: Jangan injeksi dimensi mati dari Train Pair!
                    delta_y: 0.0,
                    axiom_type: format!("CROP_WINDOW_AROUND({})", color),
                    physics_tier: 7, // Tetap masuk Tier 7 (Dimensi)
                });
            }
        }

        axioms
    }

    /// Deteksi: "Kotak berongga menjadi solid" → FILL_HOLE
    fn generate_fill_axioms(
        input_atoms: &[GestaltAtom],
        output_atoms: &[GestaltAtom],
    ) -> Vec<TopologicalMatch> {
        let mut axioms = Vec::new();

        for in_atom in input_atoms {
            if let AtomType::HollowRectangle = in_atom.atom_type {
                // Cari apakah ada solid rectangle di output dengan bbox mirip
                for out_atom in output_atoms {
                    if let AtomType::SolidRectangle = out_atom.atom_type {
                        if Self::bbox_similar(&in_atom.bounding_box, &out_atom.bounding_box) {
                            // FILL_HOLE detected!
                            let id_tensor = Self::identity_tensor();
                            let color_cond = FHRR::fractional_bind(
                                CoreSeeds::color_seed(),
                                in_atom.color as f32,
                            );

                            axioms.push(TopologicalMatch {
                                source_index: 0,
                                target_index: -1,
                                similarity: 0.95, // High confidence!
                                condition_tensor: Some(color_cond),
                                delta_spatial: id_tensor.clone(),
                                delta_semantic: id_tensor.clone(),
                                delta_x: 0.0,
                                delta_y: 0.0,
                                axiom_type: format!("FILL_HOLLOW_{}", in_atom.color),
                                physics_tier: 6, // Spawn/Fill tier
                            });
                        }
                    }
                }
            }
        }

        axioms
    }

    /// Deteksi: "Kontainer dengan konten berbeda" → RECOLOR_CONTENT
    fn generate_container_axioms(
        input_structs: &[RelationalStructure],
        output_structs: &[RelationalStructure],
    ) -> Vec<TopologicalMatch> {
        let mut axioms = Vec::new();

        for in_struct in input_structs {
            if let StructureType::ContainerWithContent = in_struct.structure_type {
                for out_struct in output_structs {
                    if let StructureType::ContainerWithContent = out_struct.structure_type {
                        // Compare bounding boxes to see if they are the same container
                        if Self::bbox_similar(&in_struct.bounding_box, &out_struct.bounding_box) {
                            axioms.push(TopologicalMatch {
                                source_index: 0,
                                target_index: -1,
                                similarity: 0.75, // Moderate confidence
                                condition_tensor: None,
                                delta_spatial: Self::identity_tensor(),
                                delta_semantic: Self::identity_tensor(),
                                delta_x: 0.0,
                                delta_y: 0.0,
                                axiom_type: "RECOLOR_CONTAINER_CONTENT".to_string(),
                                physics_tier: 0,
                            });
                        }
                    }
                }
            }
        }

        axioms
    }

    /// Deteksi: "Simetri tidak lagi simetris" → BREAK_SYMMETRY atau ENFORCE_SYMMETRY
    fn generate_symmetry_axioms(
        input_atoms: &[GestaltAtom],
        output_atoms: &[GestaltAtom],
    ) -> Vec<TopologicalMatch> {
        let mut axioms = Vec::new();

        // Check if symmetric input became asymmetric or vice versa
        let input_symmetric = input_atoms.iter().any(|a| a.symmetry_score > 0.8);
        let output_symmetric = output_atoms.iter().any(|a| a.symmetry_score > 0.8);

        if input_symmetric && !output_symmetric {
            // Generate BREAK_SYMMETRY
        } else if !input_symmetric && output_symmetric {
            // Generate ENFORCE_SYMMETRY (MIRROR_X, MIRROR_Y)
            axioms.push(TopologicalMatch {
                source_index: 0,
                target_index: -1,
                similarity: 0.8,
                condition_tensor: None,
                delta_spatial: Self::identity_tensor(),
                delta_semantic: Self::identity_tensor(),
                delta_x: 0.0,
                delta_y: 0.0,
                axiom_type: "GLOBAL_MIRROR_X".to_string(),
                physics_tier: 4,
            });
            axioms.push(TopologicalMatch {
                source_index: 0,
                target_index: -1,
                similarity: 0.8,
                condition_tensor: None,
                delta_spatial: Self::identity_tensor(),
                delta_semantic: Self::identity_tensor(),
                delta_x: 0.0,
                delta_y: 0.0,
                axiom_type: "GLOBAL_MIRROR_Y".to_string(),
                physics_tier: 4,
            });
        }

        axioms
    }

    /// Deteksi: "Grid berubah ukuran" → EXTEND_GRID atau CROP_GRID
    fn generate_grid_axioms(
        input_structs: &[RelationalStructure],
        output_structs: &[RelationalStructure],
    ) -> Vec<TopologicalMatch> {
        let mut axioms = Vec::new();

        let input_grid = input_structs
            .iter()
            .find(|s| matches!(s.structure_type, StructureType::GridPattern));
        let output_grid = output_structs
            .iter()
            .find(|s| matches!(s.structure_type, StructureType::GridPattern));

        if let (Some(ig), Some(og)) = (input_grid, output_grid) {
            let input_size = ig.atoms.len();
            let output_size = og.atoms.len();

            if output_size > input_size {
                axioms.push(TopologicalMatch {
                    source_index: 0,
                    target_index: -1,
                    similarity: 0.85,
                    condition_tensor: None,
                    delta_spatial: Self::identity_tensor(),
                    delta_semantic: Self::identity_tensor(),
                    delta_x: 0.0,
                    delta_y: 0.0,
                    axiom_type: "EXTEND_GRID_PATTERN".to_string(),
                    physics_tier: 6,
                });
            } else if output_size < input_size {
                axioms.push(TopologicalMatch {
                    source_index: 0,
                    target_index: -1,
                    similarity: 0.85,
                    condition_tensor: None,
                    delta_spatial: Self::identity_tensor(),
                    delta_semantic: Self::identity_tensor(),
                    delta_x: 0.0,
                    delta_y: 0.0,
                    axiom_type: "CROP_GRID_PATTERN".to_string(),
                    physics_tier: 7,
                });
            }
        }

        axioms
    }

    /// Deteksi pergerakan objek (sudah ada di TopologicalAligner, tapi dengan confidence lebih tinggi)
    fn generate_movement_axioms(
        input_atoms: &[GestaltAtom],
        output_atoms: &[GestaltAtom],
    ) -> Vec<TopologicalMatch> {
        // Gunakan Gestalt matching untuk tracking objek yang sama
        // (bukan hanya by color, tapi by shape similarity)
        let mut axioms = Vec::new();

        for in_atom in input_atoms {
            // Find best matching output atom by shape + color
            let best_match = output_atoms
                .iter()
                .filter(|out| out.atom_type == in_atom.atom_type)
                .min_by_key(|out| {
                    let dx = (out.center_of_mass.0 - in_atom.center_of_mass.0) as i32;
                    let dy = (out.center_of_mass.1 - in_atom.center_of_mass.1) as i32;
                    dx * dx + dy * dy // Euclidean distance squared
                });

            if let Some(out_atom) = best_match {
                let dx = out_atom.center_of_mass.0 - in_atom.center_of_mass.0;
                let dy = out_atom.center_of_mass.1 - in_atom.center_of_mass.1;

                if dx.abs() > 0.1 || dy.abs() > 0.1 {
                    let condition =
                        FHRR::fractional_bind(CoreSeeds::color_seed(), in_atom.color as f32);

                    axioms.push(TopologicalMatch {
                        source_index: 0,
                        target_index: -1,
                        similarity: 0.9,
                        condition_tensor: Some(condition),
                        delta_spatial: Self::identity_tensor(),
                        delta_semantic: Self::identity_tensor(),
                        delta_x: dx.round(),
                        delta_y: dy.round(),
                        axiom_type: format!(
                            "IF_COLOR({})_THEN_TRANS_{}_{}",
                            in_atom.color,
                            dx.round(),
                            dy.round()
                        ),
                        physics_tier: 0,
                    });
                }
            }
        }

        axioms
    }

    fn identity_tensor() -> Array1<f32> {
        let mut t = Array1::zeros(GLOBAL_DIMENSION);
        t[0] = 1.0;
        t[GLOBAL_DIMENSION - 1] = 1.0;
        t
    }

    fn bbox_similar(a: &(f32, f32, f32, f32), b: &(f32, f32, f32, f32)) -> bool {
        let threshold = 1.0;
        (a.0 - b.0).abs() < threshold
            && (a.1 - b.1).abs() < threshold
            && (a.2 - b.2).abs() < threshold
            && (a.3 - b.3).abs() < threshold
    }
}
