use std::sync::Arc;

pub const CHUNK_SIZE: usize = 64; // Ukuran blok, dioptimalkan untuk L1 Cache (64*64*f32 = 16KB)

/// Representasi sebagian kecil dari ruang Multiverse
#[derive(Clone)]
pub struct TensorChunk {
    // Kita gunakan f32 agar sesuai dengan presisi RRM (16KB per chunk)
    pub cells: [f32; CHUNK_SIZE * CHUNK_SIZE],
}

impl Default for TensorChunk {
    fn default() -> Self {
        Self::new()
    }
}

impl TensorChunk {
    pub fn new() -> Self {
        Self {
            cells: [0.0; CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

/// State alam semesta yang dipecah-pecah.
/// Memiliki array dari Arc, bukan Arc dari array.
#[derive(Clone)]
pub struct SmartMultiverseState {
    pub width_chunks: usize,
    pub height_chunks: usize,
    pub chunks: Vec<Arc<TensorChunk>>,
}

impl SmartMultiverseState {
    pub fn new(width_chunks: usize, height_chunks: usize) -> Self {
        let total_chunks = width_chunks * height_chunks;
        let mut chunks = Vec::with_capacity(total_chunks);

        // Inisialisasi awal: kita bisa menunjuk ke 1 blok kosong yang dibagikan (shared blank chunk)
        // untuk menghemat memori ekstrem saat genesis, lalu clone-on-write nanti.
        let blank_chunk = Arc::new(TensorChunk::new());
        for _ in 0..total_chunks {
            chunks.push(blank_chunk.clone());
        }

        Self {
            width_chunks,
            height_chunks,
            chunks,
        }
    }

    /// Fungsi utama untuk mutasi dengan Zero-Friction
    pub fn modify_cell(&mut self, x: usize, y: usize, new_value: f32) {
        let chunk_x = x / CHUNK_SIZE;
        let chunk_y = y / CHUNK_SIZE;
        let chunk_index = chunk_y * self.width_chunks + chunk_x;

        if chunk_index >= self.chunks.len() {
            return; // Out of bounds, abaikan atau sesuaikan dengan batas alam semesta
        }

        let local_x = x % CHUNK_SIZE;
        let local_y = y % CHUNK_SIZE;
        let local_index = local_y * CHUNK_SIZE + local_x;

        // O-MAGIC TERJADI DI SINI:
        // Jika blok masih di-share dengan universe lain (atau dengan blank_chunk),
        // Arc::make_mut akan mengkloning *hanya* blok 16KB ini.
        let chunk = Arc::make_mut(&mut self.chunks[chunk_index]);
        chunk.cells[local_index] = new_value;
    }

    /// Fungsi baca tetap secepat sebelumnya (Lock-free)
    pub fn read_cell(&self, x: usize, y: usize) -> f32 {
        let chunk_x = x / CHUNK_SIZE;
        let chunk_y = y / CHUNK_SIZE;
        let chunk_index = chunk_y * self.width_chunks + chunk_x;

        if chunk_index >= self.chunks.len() {
            return 0.0;
        }

        let local_x = x % CHUNK_SIZE;
        let local_y = y % CHUNK_SIZE;
        let local_index = local_y * CHUNK_SIZE + local_x;

        self.chunks[chunk_index].cells[local_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_structural_sharing() {
        // Buat universe 2x2 chunks (128x128 sel)
        let mut universe1 = SmartMultiverseState::new(2, 2);

        // Awalnya semua menunjuk ke blank_chunk yang sama.
        // Mari kita ubah sel (0,0) di universe 1
        universe1.modify_cell(0, 0, 42.0);

        // Cek bahwa nilainya tersimpan
        assert_eq!(universe1.read_cell(0, 0), 42.0);

        // Simulasikan clone Multiverse (MCTS branch)
        let mut universe2 = universe1.clone();

        // Universe 2 awalnya memiliki pointer Arc yang sama ke blok yang sama dengan Universe 1.
        // Jika kita mengubah chunk lain di Universe 2, chunk pertama tidak boleh disalin!
        // Mari ubah sel di chunk (1,1) yaitu di x=65, y=65
        universe2.modify_cell(65, 65, 99.0);

        // Universe 1 tidak boleh memiliki perubahan ini
        assert_eq!(universe1.read_cell(65, 65), 0.0);

        // Universe 2 memiliki perubahannya
        assert_eq!(universe2.read_cell(65, 65), 99.0);

        // Sel 0,0 tetap 42.0 di Universe 2 (karena hasil clone)
        assert_eq!(universe2.read_cell(0, 0), 42.0);

        // Test pointer (Opsional jika ingin memastikan Arc::ptr_eq)
        // chunk[0] harus sama karena kita hanya mengubah chunk[3] di universe2.
        assert!(Arc::ptr_eq(&universe1.chunks[0], &universe2.chunks[0]));

        // chunk[3] harus berbeda
        assert!(!Arc::ptr_eq(&universe1.chunks[3], &universe2.chunks[3]));
    }
}
