use crate::core::entity_manifold::EntityManifold;
use crate::perception::universal_manifold::UniversalManifold;

pub struct HologramDecoder {
    pub manifold_perceiver: UniversalManifold,
}

impl Default for HologramDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl HologramDecoder {
    pub fn new() -> Self {
        Self {
            manifold_perceiver: UniversalManifold::new(),
        }
    }

    pub fn collapse_to_grid(
        &self,
        manifold: &EntityManifold,
        width: usize,
        height: usize,
        _threshold: f32,
    ) -> Vec<Vec<i32>> {
        let mut grid = vec![vec![0; width]; height];

        for e in 0..manifold.active_count {
            if manifold.masses[e] == 0.0 {
                continue;
            }

            let cx = manifold.centers_x[e];
            let cy = manifold.centers_y[e];
            let sx = manifold.spans_x[e].max(1.0);
            let sy = manifold.spans_y[e].max(1.0);

            // Assuming cx, cy is the top-left of the BBox if it was spawned with SCALE_AND_FILL
            // Wait, SCALE_AND_FILL spawns MULTIPLE particles of size 1x1!
            // Wait, EntitySegmenter creates entities. Is it top-left or center?
            // "Pusatkan jendela" in `top_down_axiomator` suggests `centers_x` is the center.
            // Let's just draw the bounding box from `cx - sx/2` to `cx + sx/2`
            let half_w = sx / 2.0;
            let half_h = sy / 2.0;

            // Actually, in `EntitySegmenter` `centers_x` is the exact center of mass.
            // If `sx` is 3.0, `half_w` is 1.5.
            // Min x = cx - half_w. Max x = cx + half_w.

            let min_x = (cx - half_w).ceil() as i32;
            let max_x = (cx + half_w).floor() as i32;
            let min_y = (cy - half_h).ceil() as i32;
            let max_y = (cy + half_h).floor() as i32;

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < height {
                        grid[y as usize][x as usize] = manifold.tokens[e];
                    }
                }
            }
        }

        grid
    }
}
