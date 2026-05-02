use crate::core::entity_manifold::EntityManifold;
use crate::perception::hierarchical_gestalt::GestaltEngine;
use crate::perception::universal_manifold::UniversalManifold;
use ndarray::Array1;
use std::collections::HashMap;

pub struct EntitySegmenter;

struct ParsedKey {
    x: usize,
    y: usize,
    token: i32,
}

impl EntitySegmenter {
    pub fn segment_stream(
        stream: &HashMap<String, (Array1<f32>, Array1<f32>)>, // Tuple: (GlobalSpatial, Semantic)
        manifold: &mut EntityManifold,
        _similarity_threshold: f32, // Tidak dipakai di Swarm
        perceiver: &UniversalManifold,
    ) {
        let mut global_width = 1;
        let mut global_height = 1;

        let parse_key = |key: &str| -> Option<ParsedKey> {
            let parts: Vec<&str> = key.split("_t").collect();
            if parts.len() < 2 {
                return None;
            }
            let coords: Vec<&str> = parts[0].split(',').collect();
            if coords.len() < 2 {
                return None;
            }

            Some(ParsedKey {
                x: coords[0].parse().ok()?,
                y: coords[1].parse().ok()?,
                token: parts[1].parse().ok()?,
            })
        };

        // Tahap 1: Muat raw pixels ke temporary manifold (Swarm Paradigm)
        let mut temp_manifold = EntityManifold::new();
        let mut raw_idx = 0;
        manifold.ensure_scalar_capacity(stream.len());
        temp_manifold.ensure_scalar_capacity(stream.len());
        temp_manifold.ensure_tensor_capacity(stream.len());

        temp_manifold.ensure_scalar_capacity(stream.len());

        for (key, (spatial_tensor, semantic_tensor)) in stream.iter() {
            let Some(parsed) = parse_key(key) else {
                continue;
            };
            global_width = usize::max(global_width, parsed.x + 1);
            global_height = usize::max(global_height, parsed.y + 1);

            temp_manifold.ids[raw_idx] = format!("RAW_{}", raw_idx);
            temp_manifold.masses[raw_idx] = 1.0;
            temp_manifold.tokens[raw_idx] = parsed.token;
            temp_manifold.centers_x[raw_idx] = parsed.x as f32;
            temp_manifold.centers_y[raw_idx] = parsed.y as f32;

            let mut dest_sp = temp_manifold.get_spatial_tensor_mut(raw_idx);
            dest_sp.assign(spatial_tensor);

            let mut dest_sem = temp_manifold.get_semantic_tensor_mut(raw_idx);
            dest_sem.assign(semantic_tensor);

            raw_idx += 1;
        }

        temp_manifold.global_width = global_width as f32;
        temp_manifold.global_height = global_height as f32;
        temp_manifold.active_count = raw_idx;

        // Tahap 2: Ekstraksi Gestalt (Objek Utuh) dari piksel mentah
        let atoms = GestaltEngine::extract_atoms(&temp_manifold);

        // Tahap 3: Map Gestalt ke EntityManifold aktual
        let mut manifold_idx = 0;
        let mut entity_counter = 1;

        manifold.ensure_scalar_capacity(atoms.len());

        for atom in atoms {
            manifold.ids[manifold_idx] = format!("OBJ_{}", entity_counter);
            entity_counter += 1;

            manifold.masses[manifold_idx] = atom.pixel_count as f32;
            manifold.tokens[manifold_idx] = atom.color;
            manifold.centers_x[manifold_idx] = atom.center_of_mass.0;
            manifold.centers_y[manifold_idx] = atom.center_of_mass.1;

            manifold.spans_x[manifold_idx] = atom.bounding_box.2 - atom.bounding_box.0 + 1.0;
            manifold.spans_y[manifold_idx] = atom.bounding_box.3 - atom.bounding_box.1 + 1.0;

            // Rata-rata spatial tensor (CoM)
            let mut avg_spatial = Array1::<f32>::zeros(crate::core::config::GLOBAL_DIMENSION);
            for &idx in &atom.component_indices {
                let sp = temp_manifold.get_spatial_tensor_mut(idx);
                avg_spatial += &sp;
            }
            if atom.pixel_count > 0 {
                avg_spatial /= atom.pixel_count as f32;
            }

            let mut dest_sp = manifold.get_spatial_tensor_mut(manifold_idx);
            dest_sp.assign(&avg_spatial);

            // Shape tensor
            let local_shape = perceiver.build_local_shape_tensor(
                manifold.spans_x[manifold_idx],
                manifold.spans_y[manifold_idx],
            );
            let mut dest_shape = manifold.get_shape_tensor_mut(manifold_idx);
            dest_shape.assign(&local_shape);

            // Semantic tensor (dari warna)
            let sem_tensor = crate::core::fhrr::FHRR::fractional_bind(
                crate::core::core_seeds::CoreSeeds::color_seed(),
                atom.color as f32,
            );
            let mut dest_sem = manifold.get_semantic_tensor_mut(manifold_idx);
            dest_sem.assign(&sem_tensor);

            manifold_idx += 1;
        }

        manifold.global_width = global_width as f32;
        manifold.global_height = global_height as f32;
        manifold.active_count = manifold_idx;
    }
}
