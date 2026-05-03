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
        _threshold: f32, // Tidak dipakai untuk Partikel murni, Z-Buffer 1.0 murni.
    ) -> Vec<Vec<i32>> {
        let mut grid = vec![vec![0; width]; height];

        // SWARM COLLAPSE (Lossless Absolute Collapse)
        // Kita tidak lagi memutar Sinar Probe (Superposisi) untuk menebak keberadaan.
        // Karena ini murni kumpulan Partikel Individu (Piksel),
        // kita langsung render koordinat absolut mereka di layar!

        for e in 0..manifold.active_count {
            if manifold.masses[e] == 0.0 {
                continue;
            }

            let center_x = manifold.centers_x[e].round() as i32;
            let center_y = manifold.centers_y[e].round() as i32;

            if center_x >= 0 && (center_x as usize) < width && center_y >= 0 && (center_y as usize) < height
            {
                // Untuk Swarm Kuantum dasar, token warna sudah tersimpan utuh.
                // Jika ingin ekstraksi Fasa Semantik bisa panggil Probe Warna.
                // Tapi untuk efisiensi absolut, token asli sudah tersedia di manifold!
                grid[center_y as usize][center_x as usize] = manifold.tokens[e];
            }
        }

        grid
    }
}
