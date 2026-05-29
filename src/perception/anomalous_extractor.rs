use crate::core::entity_manifold::EntityManifold;
use crate::perception::hierarchical_gestalt::{AtomType, GestaltEngine};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct AnomalousExtractor;

impl Default for AnomalousExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl AnomalousExtractor {
    pub fn new() -> Self {
        Self
    }

    ///
    /// # Errors
    /// Returns a `String` error if extraction fails (though currently it always succeeds).
    pub fn execute(&self, state: &EntityManifold) -> Result<EntityManifold, String> {
        let new_state = extract_anomalous_quadrant(state);
        Ok(new_state)
    }
}

pub fn extract_anomalous_quadrant(state: &EntityManifold) -> EntityManifold {
    if state.active_count == 0 {
        return state.clone();
    }

    let w = state.global_width as usize;
    let h = state.global_height as usize;
    let bg_color = 0;

    if w > 0 && h > 0 {
        let mut grid = vec![vec![0; w]; h];
        for i in 0..state.active_count {
            if state.masses[i] > 0.0 {
                let cx = state.centers_x[i].round() as usize;
                let cy = state.centers_y[i].round() as usize;
                if cx < w && cy < h {
                    grid[cy][cx] = state.tokens[i];
                }
            }
        }

        // HEURISTIC 1: Periodic Extrapolation (Dynamic Stride Detection)
        let mut best_period = None;
        let mut best_mapping = std::collections::HashMap::new();

        for k in 2..=(std::cmp::max(w, h) / 2).max(4) {
            let mut is_valid = true;
            let mut mapping = std::collections::HashMap::new();
            let mut has_elements = false;

            for r in 0..h {
                for c in 0..w {
                    let val = grid[r][c];
                    if val != bg_color {
                        has_elements = true;
                        let key = (r + c) % k;
                        if let Some(&existing) = mapping.get(&key) {
                            if existing != val {
                                is_valid = false;
                                break;
                            }
                        } else {
                            mapping.insert(key, val);
                        }
                    }
                }
                if !is_valid {
                    break;
                }
            }

            if has_elements && is_valid && mapping.len() > 1 && mapping.len() <= k {
                best_period = Some(k);
                best_mapping = mapping;
                break;
            }
        }

        if let Some(k) = best_period {
            let mut out_state = EntityManifold::new();
            out_state.global_width = state.global_width;
            out_state.global_height = state.global_height;
            let mut idx = 0;
            for r in 0..h {
                for c in 0..w {
                    if let Some(&val) = best_mapping.get(&((r + c) % k)) {
                        out_state.ensure_scalar_capacity(idx + 1);
                        out_state.masses[idx] = 1.0;
                        out_state.tokens[idx] = val;
                        out_state.centers_x[idx] = c as f32;
                        out_state.centers_y[idx] = r as f32;
                        out_state.spans_x[idx] = 1.0;
                        out_state.spans_y[idx] = 1.0;
                        idx += 1;
                    }
                }
            }
            out_state.active_count = idx;
            return out_state;
        }

        // HEURISTIC 2: Fractal Isomorphism (Partitioned Grid Reasoning)
        let mut possible_separators = std::collections::HashSet::new();
        for r in 0..h {
            let mut row_colors = std::collections::HashSet::new();
            for c in 0..w {
                row_colors.insert(grid[r][c]);
            }
            if row_colors.len() == 1 {
                if let Some(&color) = row_colors.iter().next() {
                    possible_separators.insert(color);
                }
            }
        }

        for sep in possible_separators {
            if sep == bg_color {
                continue;
            }
            let mut r_sep = Vec::new();
            for r in 0..h {
                if grid[r].iter().all(|&x| x == sep) {
                    r_sep.push(r);
                }
            }

            let mut c_sep = Vec::new();
            for c in 0..w {
                if (0..h).all(|r| grid[r][c] == sep) {
                    c_sep.push(c);
                }
            }

            if !r_sep.is_empty() && !c_sep.is_empty() {
                let mut r_bounds = vec![-1];
                for &r in &r_sep {
                    r_bounds.push(r as i32);
                }
                r_bounds.push(h as i32);

                let mut c_bounds = vec![-1];
                for &c in &c_sep {
                    c_bounds.push(c as i32);
                }
                c_bounds.push(w as i32);

                struct Quad {
                    r_idx: usize,
                    c_idx: usize,
                    pixels: Vec<(usize, usize, i32)>,
                    r_start: usize,
                    r_end: usize,
                    c_start: usize,
                    c_end: usize,
                }
                let mut quads = Vec::new();

                for (r_idx, _) in r_bounds
                    .iter()
                    .enumerate()
                    .take(r_bounds.len().saturating_sub(1))
                {
                    for (c_idx, _) in c_bounds
                        .iter()
                        .enumerate()
                        .take(c_bounds.len().saturating_sub(1))
                    {
                        let r_start = (r_bounds[r_idx] + 1) as usize;
                        let r_end = r_bounds[r_idx + 1] as usize;
                        let c_start = (c_bounds[c_idx] + 1) as usize;
                        let c_end = c_bounds[c_idx + 1] as usize;
                        if r_start >= r_end || c_start >= c_end {
                            continue;
                        }

                        let mut pixels = Vec::new();
                        for r in r_start..r_end {
                            for c in c_start..c_end {
                                if grid[r][c] != bg_color && grid[r][c] != sep {
                                    pixels.push((r - r_start, c - c_start, grid[r][c]));
                                }
                            }
                        }
                        quads.push(Quad {
                            r_idx,
                            c_idx,
                            pixels,
                            r_start,
                            r_end,
                            c_start,
                            c_end,
                        });
                    }
                }

                // Sub-Heuristic: The "Incomplete Mapping"
                let mut base_quad_idx = None;
                let mut min_pixels = usize::MAX;
                for (i, q) in quads.iter().enumerate() {
                    if !q.pixels.is_empty() && q.pixels.len() < min_pixels {
                        min_pixels = q.pixels.len();
                        base_quad_idx = Some(i);
                    }
                }

                if let Some(idx) = base_quad_idx {
                    let base_quad = &quads[idx];
                    let mut out_state = EntityManifold::new();
                    out_state.global_width = state.global_width;
                    out_state.global_height = state.global_height;
                    let mut out_idx = 0;

                    for r in 0..h {
                        for c in 0..w {
                            if r_sep.contains(&r) || c_sep.contains(&c) {
                                out_state.ensure_scalar_capacity(out_idx + 1);
                                out_state.masses[out_idx] = 1.0;
                                out_state.tokens[out_idx] = sep;
                                out_state.centers_x[out_idx] = c as f32;
                                out_state.centers_y[out_idx] = r as f32;
                                out_state.spans_x[out_idx] = 1.0;
                                out_state.spans_y[out_idx] = 1.0;
                                out_idx += 1;
                            }
                        }
                    }

                    for &(pr, pc, v) in &base_quad.pixels {
                        if let Some(target_q) =
                            quads.iter().find(|q| q.r_idx == pr && q.c_idx == pc)
                        {
                            for r in target_q.r_start..target_q.r_end {
                                for c in target_q.c_start..target_q.c_end {
                                    out_state.ensure_scalar_capacity(out_idx + 1);
                                    out_state.masses[out_idx] = 1.0;
                                    out_state.tokens[out_idx] = v;
                                    out_state.centers_x[out_idx] = c as f32;
                                    out_state.centers_y[out_idx] = r as f32;
                                    out_state.spans_x[out_idx] = 1.0;
                                    out_state.spans_y[out_idx] = 1.0;
                                    out_idx += 1;
                                }
                            }
                        }
                    }
                    out_state.active_count = out_idx;
                    return out_state;
                }
            }
        }
    }

    // ========================================================
    // FASE 1: MICRO SCALE (Semantic & Geometric Detection)
    // ========================================================
    // RRM "Mata Struktural" - Menganalisa Topologi Lanskap via GestaltEngine
    // Menciptakan "Medan Gravitasi" yang ditarik oleh anomali bentuk / simetri

    let atoms = GestaltEngine::extract_atoms(state);

    // Kelompokkan berdasarkan AtomType (Signature Geometris)
    let mut shape_counts = HashMap::new();
    let mut symmetry_scores = HashMap::new();

    for (i, atom) in atoms.iter().enumerate() {
        let signature = match atom.atom_type {
            AtomType::SolidRectangle => "SolidRectangle",
            AtomType::HollowRectangle => "HollowRectangle",
            AtomType::HorizontalLine => "HorizontalLine",
            AtomType::VerticalLine => "VerticalLine",
            AtomType::DiagonalLine => "DiagonalLine",
            AtomType::LShape => "LShape",
            AtomType::TShape => "TShape",
            AtomType::CrossShape => "CrossShape",
            AtomType::Scatter => "Scatter",
            AtomType::SinglePixel => "SinglePixel",
        };

        *shape_counts.entry(signature).or_insert(0) += 1;
        symmetry_scores.insert(i, atom.symmetry_score);
    }

    let mut minority_shape = "None";
    if let Some((&shape, _)) = shape_counts
        .iter()
        .filter(|&(_, &c)| c > 0)
        .min_by_key(|&(_, c)| c)
    {
        minority_shape = shape;
    }

    // Cari titik pusat anomali gravitasi (titik terdalam dari sumur probabilitas)
    let mut anomaly_x = 0.0;
    let mut anomaly_y = 0.0;
    let mut anomaly_found = false;

    // 1. Prioritaskan bentuk minoritas (misal: ada 1 LShape di antara 3 SolidRectangle)
    for atom in &atoms {
        let signature = match atom.atom_type {
            AtomType::SolidRectangle => "SolidRectangle",
            AtomType::HollowRectangle => "HollowRectangle",
            AtomType::HorizontalLine => "HorizontalLine",
            AtomType::VerticalLine => "VerticalLine",
            AtomType::DiagonalLine => "DiagonalLine",
            AtomType::LShape => "LShape",
            AtomType::TShape => "TShape",
            AtomType::CrossShape => "CrossShape",
            AtomType::Scatter => "Scatter",
            AtomType::SinglePixel => "SinglePixel",
        };

        if signature == minority_shape && shape_counts[minority_shape] == 1 {
            // Anomali struktural mutlak!
            anomaly_x = atom.center_of_mass.0;
            anomaly_y = atom.center_of_mass.1;
            anomaly_found = true;
            break;
        }
    }

    // 2. Jika tidak ada minoritas bentuk, cari yang symmetry_score nya paling anjlok
    if !anomaly_found && !atoms.is_empty() {
        let mut min_sym = 1.0;
        let mut best_atom = None;
        for atom in &atoms {
            // Abaikan SinglePixel karena pasti simetris 1.0
            if atom.atom_type != AtomType::SinglePixel && atom.symmetry_score < min_sym {
                min_sym = atom.symmetry_score;
                best_atom = Some(atom);
            }
        }

        // Jika ada anomali simetri (misal 0.4 vs 1.0)
        if let Some(atom) = best_atom {
            if min_sym < 0.8 {
                anomaly_x = atom.center_of_mass.0;
                anomaly_y = atom.center_of_mass.1;
                anomaly_found = true;
            }
        }
    }

    // 3. Fallback ke Warna jika lanskap geometris datar (seperti ARC 2dc579da yang hanya dot 1 piksel)
    let bg_color = 0;
    let mut grid_map = HashMap::new();
    let mut color_counts = HashMap::new();

    for i in 0..state.active_count {
        if state.masses[i] > 0.0 && state.tokens[i] != bg_color {
            let cx = state.centers_x[i].round() as i32;
            let cy = state.centers_y[i].round() as i32;
            let w = state.spans_x[i].round() as i32;
            let h = state.spans_y[i].round() as i32;

            if w <= 1 && h <= 1 {
                grid_map.insert((cx, cy), i);
            } else {
                let left = cx - (w - 1) / 2;
                let top = cy - (h - 1) / 2;
                for dx in 0..w {
                    for dy in 0..h {
                        grid_map.insert((left + dx, top + dy), i);
                    }
                }
            }
            *color_counts.entry(state.tokens[i]).or_insert(0) += 1;
        }
    }

    if grid_map.is_empty() {
        return state.clone();
    }

    // ========================================================
    // FASE 2: NANO SCALE (Relational Alignment / Grouping)
    // ========================================================
    // Still using flood-fill for rigid bounds, but now informed by Micro scale

    let mut visited = HashSet::new();
    let mut quadrants = Vec::new();
    let dirs = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    for (&(sx, sy), _) in grid_map.iter() {
        if !visited.contains(&(sx, sy)) {
            let mut queue = VecDeque::new();
            queue.push_back((sx, sy));
            visited.insert((sx, sy));

            let mut component_pixels = Vec::new();
            let mut component_colors = HashSet::new();
            let mut component_shapes = HashSet::new();

            while let Some((cx, cy)) = queue.pop_front() {
                if let Some(&idx) = grid_map.get(&(cx, cy)) {
                    component_pixels.push((cx, cy, idx));
                    component_colors.insert(state.tokens[idx]);

                    // Map back to atom to get shape
                    if let Some(atom) = atoms.iter().find(|a| a.component_indices.contains(&idx)) {
                        let signature = match atom.atom_type {
                            AtomType::SolidRectangle => "SolidRectangle",
                            AtomType::HollowRectangle => "HollowRectangle",
                            AtomType::HorizontalLine => "HorizontalLine",
                            AtomType::VerticalLine => "VerticalLine",
                            AtomType::LShape => "LShape",
                            AtomType::TShape => "TShape",
                            AtomType::CrossShape => "CrossShape",
                            AtomType::Scatter => "Scatter",
                            AtomType::SinglePixel => "SinglePixel",
                            AtomType::DiagonalLine => "DiagonalLine",
                        };
                        component_shapes.insert(signature);
                    }
                }

                for &(dx, dy) in &dirs {
                    let nx = cx + dx;
                    let ny = cy + dy;
                    if grid_map.contains_key(&(nx, ny)) && !visited.contains(&(nx, ny)) {
                        visited.insert((nx, ny));
                        queue.push_back((nx, ny));
                    }
                }
            }

            quadrants.push((component_pixels, component_colors, component_shapes));
        }
    }

    if quadrants.len() <= 1 {
        return state.clone();
    }

    let max_size = quadrants.iter().map(|(p, _, _)| p.len()).max().unwrap_or(0);
    let valid_quadrants: Vec<_> = quadrants
        .into_iter()
        .filter(|(p, _, _)| p.len() as f32 > max_size as f32 * 0.2)
        .collect();

    if valid_quadrants.len() <= 1 {
        return state.clone();
    }

    let mut global_color_counts = HashMap::new();
    for (_, colors, _) in &valid_quadrants {
        for &color in colors.iter() {
            *global_color_counts.entry(color).or_insert(0) += 1;
        }
    }

    let mut best_idx = 0;
    let mut best_score = -1;

    for (idx, (_, colors, shapes)) in valid_quadrants.iter().enumerate() {
        let mut score = 0;

        // 1. Color Anomaly (Micro scale logic)
        for &color in colors.iter() {
            if *global_color_counts.get(&color).unwrap_or(&0) == 1 {
                score += 10;
            }
        }
        score += colors.len() as i32;

        // 2. Shape Anomaly (Micro scale logic)
        if shapes.contains(minority_shape) && minority_shape != "None" {
            score += 15; // Stronger signal for unique topological shape
        }

        // 3. Symmetry Gravity / Topologi Ekstrem (Micro)
        // Di mana anomaly_found disematkan!
        if anomaly_found {
            let (best_pixels, _, _) = &valid_quadrants[idx];
            let mut q_min_x = i32::MAX;
            let mut q_max_x = i32::MIN;
            let mut q_min_y = i32::MAX;
            let mut q_max_y = i32::MIN;

            for &(cx, cy, _) in best_pixels.iter() {
                if cx < q_min_x {
                    q_min_x = cx;
                }
                if cx > q_max_x {
                    q_max_x = cx;
                }
                if cy < q_min_y {
                    q_min_y = cy;
                }
                if cy > q_max_y {
                    q_max_y = cy;
                }
            }

            let ax = anomaly_x.round() as i32;
            let ay = anomaly_y.round() as i32;

            // Tolerasni bounding box
            if ax >= q_min_x - 1 && ax <= q_max_x + 1 && ay >= q_min_y - 1 && ay <= q_max_y + 1 {
                score += 50; // Medan gravitasi asimetris menarik tebakan ke kuadran ini
            }
        }

        if score > best_score {
            best_score = score;
            best_idx = idx;
        }
    }

    let (best_pixels, _, _) = &valid_quadrants[best_idx];

    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;

    for &(cx, cy, _) in best_pixels.iter() {
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

    // ========================================================
    // FASE 3: PICO SCALE (Geometric Transform)
    // ========================================================
    // ... Placeholder untuk SymmetryGroup / Rotate ...

    // ========================================================
    // FASE 4: FEMTO SCALE (Absolute Crop & Coordinate Normalization)
    // ========================================================
    let new_w = (max_x - min_x) as f32 + 1.0;
    let new_h = (max_y - min_y) as f32 + 1.0;

    let mut new_state = EntityManifold::new();
    new_state.global_width = new_w;
    new_state.global_height = new_h;

    let mut copied = 0;
    let mut processed_idx = HashSet::new();

    for &(_cx, _cy, idx) in best_pixels.iter() {
        if !processed_idx.contains(&idx) {
            processed_idx.insert(idx);

            new_state.ensure_scalar_capacity(copied + 1);

            new_state.masses[copied] = state.masses[idx];
            new_state.tokens[copied] = state.tokens[idx];

            new_state.centers_x[copied] = state.centers_x[idx] - min_x as f32;
            new_state.centers_y[copied] = state.centers_y[idx] - min_y as f32;

            new_state.spans_x[copied] = state.spans_x[idx];
            new_state.spans_y[copied] = state.spans_y[idx];

            copied += 1;
        }
    }

    new_state.active_count = copied;
    new_state
}
